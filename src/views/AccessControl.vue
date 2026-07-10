<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";

const enabled = ref(false);
const ips = ref<string[]>([]);
const newIp = ref("");
const busy = ref(false);
const error = ref("");
const loaded = ref(false);

async function load() {
  error.value = "";
  try {
    const res = await api.getIpWhitelist();
    enabled.value = res.enabled;
    ips.value = res.ips;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    loaded.value = true;
  }
}

async function toggleEnabled() {
  busy.value = true;
  error.value = "";
  try {
    const res = await api.setIpWhitelist({ enabled: !enabled.value });
    enabled.value = res.enabled;
    ips.value = res.ips;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    busy.value = false;
  }
}

async function addIp() {
  const ip = newIp.value.trim();
  if (!ip) return;
  busy.value = true;
  error.value = "";
  try {
    const res = await api.setIpWhitelist({ ip });
    enabled.value = res.enabled;
    ips.value = res.ips;
    newIp.value = "";
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    busy.value = false;
  }
}

async function removeIp(ip: string) {
  busy.value = true;
  error.value = "";
  try {
    const res = await api.setIpWhitelist({ remove: ip });
    enabled.value = res.enabled;
    ips.value = res.ips;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    busy.value = false;
  }
}

onMounted(load);
</script>

<template>
  <p v-if="error" class="notice bad">{{ error }}</p>

  <template v-if="loaded">
  <section class="panel">
    <div class="panel-heading">
      <p class="section-number">01</p>
      <div>
        <h2>IP 白名单</h2>
        <p>开启后，仅白名单中的 IP 可以访问本服务（WebUI 及 /v1 代理）。127.0.0.1 始终放行。</p>
      </div>
    </div>

    <div class="account-row">
      <span :class="['state-line', enabled ? 'ok' : 'warn']">
        白名单：{{ enabled ? "已开启" : "已关闭" }}
      </span>
      <button class="line-action" :disabled="busy" @click="toggleEnabled">
        {{ enabled ? "关闭白名单" : "开启白名单" }}
      </button>
    </div>
    <p v-if="!enabled" class="notice warn">
      当前白名单未开启，仅 127.0.0.1 可访问（白名单需通过 --host 0.0.0.0 启动）
    </p>
  </section>

  <section class="panel">
    <div class="panel-heading compact">
      <p class="section-number">02</p>
      <div>
        <h2>添加 IP</h2>
        <p>输入允许访问的设备 IP 地址（IPv4）。</p>
      </div>
    </div>
    <div class="two-factor-row">
      <label>
        <span>IP 地址</span>
        <input v-model="newIp" placeholder="例如：192.168.1.100" @keyup.enter="addIp" />
      </label>
      <button class="primary-action" :disabled="busy || !newIp.trim()" @click="addIp">添加</button>
    </div>
  </section>

  <section class="panel">
    <div class="panel-heading compact">
      <p class="section-number">03</p>
      <div>
        <h2>白名单列表</h2>
        <p>{{ ips.length }} 个 IP</p>
      </div>
    </div>
    <p v-if="!ips.length" class="notice">白名单为空，添加 IP 后才能生效。</p>
    <div v-else class="model-table">
      <div v-for="ip in ips" :key="ip" class="ip-row">
        <code>{{ ip }}</code>
        <button class="line-action danger" :disabled="busy" @click="removeIp(ip)">删除</button>
      </div>
    </div>
  </section>
  </template>
</template>

<style scoped>
.ip-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.6rem 0;
  border-bottom: 1px solid var(--hairline, rgba(128, 128, 128, 0.2));
}
.ip-row code {
  font-size: 0.95em;
}
</style>
