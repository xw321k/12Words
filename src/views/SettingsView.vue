<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVaultStore } from '../stores/vault'
import { useLocaleStore } from '../stores/locale'
import { messages } from '../i18n'

const router = useRouter()
const vault = useVaultStore()
const loc = useLocaleStore()
const exportMsg = ref('')

function t(key: string, params?: Record<string, string | number>): string {
  const msg = messages[loc.locale]?.[key]
  if (!msg) return key
  if (!params) return msg
  return msg.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? ''))
}

async function handleExport() {
  exportMsg.value = ''
  try {
    const msg = await vault.exportBackup()
    exportMsg.value = msg
  } catch (e: any) {
    exportMsg.value = String(e)
  }
  setTimeout(() => { exportMsg.value = '' }, 3000)
}
</script>

<template>
  <div class="flex-1 flex flex-col overflow-hidden" :style="{ background: 'var(--color-surface-secondary)' }">
    <div class="h-14 flex items-center px-6 border-b flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface)' }">
      <h1 class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.title') }}</h1>
    </div>
    <div class="flex-1 overflow-y-auto px-6 py-8">
      <div class="max-w-lg mx-auto space-y-4">
        <!-- Language -->
        <div class="rounded-xl p-5" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.language') }}</h2>
          <div class="flex gap-2">
            <button @click="loc.setLocale('zh')"
              class="px-4 py-1.5 rounded-md text-xs font-medium border cursor-pointer transition-colors"
              :style="{
                borderColor: loc.locale === 'zh' ? 'var(--color-accent)' : 'var(--color-border)',
                color: loc.locale === 'zh' ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                background: loc.locale === 'zh' ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'transparent',
              }">中文</button>
            <button @click="loc.setLocale('en')"
              class="px-4 py-1.5 rounded-md text-xs font-medium border cursor-pointer transition-colors"
              :style="{
                borderColor: loc.locale === 'en' ? 'var(--color-accent)' : 'var(--color-border)',
                color: loc.locale === 'en' ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                background: loc.locale === 'en' ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'transparent',
              }">English</button>
          </div>
        </div>

        <!-- Export only -->
        <div class="rounded-xl p-5" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.backup') }}</h2>
          <div class="space-y-3">
            <div class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">
              {{ t('settings.backupDesc') }}
            </div>
            <button @click="handleExport"
              class="w-full py-2 rounded-lg text-xs font-medium border cursor-pointer transition-colors duration-100"
              :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">
              {{ t('settings.exportBtn') }}
            </button>
            <p v-if="exportMsg" class="text-xs" :style="{ color: exportMsg.includes('成功') || exportMsg.includes('successful') ? 'var(--color-success)' : 'var(--color-danger)' }">
              {{ exportMsg }}
            </p>
          </div>
        </div>
        <div class="rounded-xl p-5" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.account') }}</h2>
          <div class="space-y-2">
            <div class="flex justify-between">
              <span class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">User ID</span>
              <span class="text-xs font-mono select-all" :style="{ color: 'var(--color-text-primary)' }">{{ vault.userId.slice(0, 16) }}...</span>
            </div>
            <div class="flex justify-between">
              <span class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">{{ t('settings.entries', { count: vault.entryCount }) }}</span>
              <span class="text-xs" :style="{ color: 'var(--color-text-primary)' }">{{ vault.entryCount }} {{ loc.locale === 'zh' ? '项' : 'entries' }}</span>
            </div>
          </div>
        </div>
        <div class="rounded-xl p-5" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.security') }}</h2>
          <div class="space-y-3">
            <div class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">
              <p>{{ t('settings.securityDesc') }}</p>
            </div>
            <button @click="vault.lock(); router.push('/onboarding')"
              class="px-4 py-1.5 rounded-md text-xs border cursor-pointer transition-colors duration-100"
              :style="{ borderColor: 'var(--color-danger)', color: 'var(--color-danger)', background: 'transparent' }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">
              {{ t('settings.lock') }}
            </button>
          </div>
        </div>
        <div class="rounded-xl p-5" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">{{ t('settings.about') }}</h2>
          <div class="text-xs space-y-2" :style="{ color: 'var(--color-text-tertiary)' }">
            <p :style="{ color: 'var(--color-text-primary)' }">12Words v0.1.0</p>
            <p class="text-[11px]">{{ t('settings.aboutLine1') }}</p>
            <p class="text-[11px]">{{ t('settings.aboutLine2') }}</p>
            <div class="h-px my-2" :style="{ background: 'var(--color-border)' }" />
            <p class="text-[11px]" :style="{ color: 'var(--color-text-secondary)' }">{{ t('settings.aboutAuthor') }}</p>
            <button @click="vault.openUrl('https://github.com/xw321k/12words')"
              class="inline-flex items-center gap-1.5 text-[11px] font-medium border-none cursor-pointer bg-transparent px-0 hover:underline"
              :style="{ color: 'var(--color-accent)' }">
              <span>🔗</span> {{ t('settings.aboutOpenSource') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
