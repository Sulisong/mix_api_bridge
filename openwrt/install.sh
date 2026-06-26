#!/bin/sh
# On-router installer for the Mimo Bridge OpenWrt bundle.
#
# Run this ON THE ROUTER, from inside an extracted bundle directory that
# contains ./mix_api_bridge (the binary) and ./files + ./luci trees.
#
#   tar -xzf mix_api_bridge_openwrt_aarch64.tar.gz
#   cd mix_api_bridge_openwrt_*
#   sh install.sh
#
# Re-running is safe (idempotent); it upgrades the binary and refreshes LuCI.
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
BIN_SRC="$HERE/mix_api_bridge"

echo "==> Mimo Bridge OpenWrt installer"

[ -f "$BIN_SRC" ] || { echo "!! missing binary: $BIN_SRC" >&2; exit 1; }

# --- sanity: arch check -----------------------------------------------------
case "$(uname -m)" in
	aarch64|arm64) : ;;
	*) echo "!! warning: router arch is $(uname -m), bundle is aarch64. Continuing anyway." >&2 ;;
esac

# --- 1. binary --------------------------------------------------------------
echo "==> installing /usr/bin/mix_api_bridge"
# stop a running instance so we can overwrite a busy file
[ -x /etc/init.d/mix_api_bridge ] && /etc/init.d/mix_api_bridge stop 2>/dev/null || true
cp -f "$BIN_SRC" /usr/bin/mix_api_bridge
chmod +x /usr/bin/mix_api_bridge

# --- 2. init + config -------------------------------------------------------
echo "==> installing init script + UCI config"
cp -f "$HERE/files/etc/init.d/mix_api_bridge" /etc/init.d/mix_api_bridge
chmod +x /etc/init.d/mix_api_bridge
# don't clobber an existing user config
if [ ! -f /etc/config/mix_api_bridge ]; then
	cp -f "$HERE/files/etc/config/mix_api_bridge" /etc/config/mix_api_bridge
else
	echo "   keeping existing /etc/config/mix_api_bridge"
fi

# --- 3. LuCI app ------------------------------------------------------------
echo "==> installing LuCI app"
mkdir -p /www/luci-static/resources/view/mix
cp -f "$HERE/luci/htdocs/luci-static/resources/view/mix/overview.js" \
	/www/luci-static/resources/view/mix/overview.js
mkdir -p /usr/share/luci/menu.d /usr/share/rpcd/acl.d
cp -f "$HERE/luci/root/usr/share/luci/menu.d/luci-app-mix.json" \
	/usr/share/luci/menu.d/luci-app-mix.json
cp -f "$HERE/luci/root/usr/share/rpcd/acl.d/luci-app-mix.json" \
	/usr/share/rpcd/acl.d/luci-app-mix.json

# --- 4. enable + start ------------------------------------------------------
echo "==> enabling + starting service"
/etc/init.d/mix_api_bridge enable 2>/dev/null || true
/etc/init.d/mix_api_bridge start

# --- 5. refresh LuCI/rpcd caches -------------------------------------------
echo "==> refreshing LuCI / rpcd caches"
rm -f /tmp/luci-indexcache 2>/dev/null || true
rm -rf /tmp/luci-modulecache 2>/dev/null || true
/etc/init.d/rpcd reload 2>/dev/null || true
/etc/init.d/uhttpd restart 2>/dev/null || true

PORT="$(uci -q get mix_api_bridge.main.port 2>/dev/null || echo 8765)"
echo
echo "==> done."
echo "    Service:  /etc/init.d/mix_api_bridge {start|stop|restart|enable}"
echo "    WebUI:    http://<router-ip>:$PORT/"
echo "    LuCI:     Services -> Mimo Bridge"
echo
echo "    Persist across sysupgrade by adding these to /etc/sysupgrade.conf:"
echo "      /etc/config/mix_api_bridge"
echo "      /etc/mix_api_bridge/"
