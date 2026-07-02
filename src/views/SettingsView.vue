<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useVaultStore } from '../stores/vault'

const router = useRouter()
const vault = useVaultStore()
</script>

<template>
  <div class="flex-1 flex flex-col overflow-hidden" :style="{ background: 'var(--color-surface-secondary)' }">
    <!-- Header -->
    <div
      class="h-14 flex items-center px-6 border-b flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface)' }"
    >
      <h1 class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">
        设置
      </h1>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-8">
      <div class="max-w-lg mx-auto space-y-4">
        <!-- Account info -->
        <div
          class="rounded-xl p-5"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">
            账户信息
          </h2>
          <div class="space-y-2">
            <div class="flex justify-between">
              <span class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">User ID</span>
              <span class="text-xs font-mono select-all" :style="{ color: 'var(--color-text-primary)' }">
                {{ vault.userId.slice(0, 16) }}...
              </span>
            </div>
            <div class="flex justify-between">
              <span class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">密码条目</span>
              <span class="text-xs" :style="{ color: 'var(--color-text-primary)' }">{{ vault.entryCount }} 项</span>
            </div>
          </div>
        </div>

        <!-- Security -->
        <div
          class="rounded-xl p-5"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">
            安全
          </h2>
          <div class="space-y-3">
            <div class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">
              <p>数据使用 AES-256-GCM 加密存储，密钥由你的 12 位助记词衍生。</p>
              <p class="mt-1">加密文件保存在本地应用数据目录。</p>
            </div>
            <button
              @click="vault.lock(); router.push('/onboarding')"
              class="px-4 py-1.5 rounded-md text-xs border cursor-pointer transition-colors duration-100"
              :style="{
                borderColor: 'var(--color-danger)',
                color: 'var(--color-danger)',
                background: 'transparent',
              }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
            >
              锁定密码库
            </button>
          </div>
        </div>

        <!-- About -->
        <div
          class="rounded-xl p-5"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">
            关于
          </h2>
          <div class="text-xs space-y-1" :style="{ color: 'var(--color-text-tertiary)' }">
            <p>12Words v0.1.0</p>
            <p>零知识密码管理器</p>
            <p>基于 BIP39 + AES-256-GCM</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
