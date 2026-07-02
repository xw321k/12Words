<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVaultStore } from '../stores/vault'

const router = useRouter()
const vault = useVaultStore()
const exportMsg = ref('')
const importMsg = ref('')
const importing = ref(false)

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

async function handleImport() {
  importMsg.value = ''
  importing.value = true
  try {
    await vault.importBackup()
    importMsg.value = `导入成功！共 ${vault.entryCount} 条密码记录。`
  } catch (e: any) {
    importMsg.value = String(e)
  }
  importing.value = false
  setTimeout(() => { importMsg.value = '' }, 4000)
}
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
        <!-- Zero-trust announcement -->
        <div
          class="rounded-xl p-5"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-accent)' }"
        >
          <div class="flex items-start gap-3">
            <span class="text-base flex-shrink-0 mt-0.5">🛡️</span>
            <div class="text-xs space-y-2" :style="{ color: 'var(--color-text-secondary)' }">
              <p class="font-semibold text-sm" :style="{ color: 'var(--color-text-primary)' }">
                12Words 提示
              </p>
              <p>
                本软件为 <strong>100% 纯本地零信任架构</strong>，我们不设服务器，绝不上传您的任何隐私。
              </p>
              <p>
                请注意：<strong>12 个助记词</strong>是解密您密码箱的唯一钥匙，而本地加密文件（<code>vault.encrypted</code>）是您数据的唯一载体。<strong>二者缺一不可</strong>。
              </p>
              <p>建议您：</p>
              <ul class="list-disc pl-4 space-y-1">
                <li>抄写并妥善保管好这 12 个助记词。</li>
                <li>定期使用软件的「一键备份」功能，将加密文件导出至您的 U 盘或私人云盘中。</li>
              </ul>
            </div>
          </div>
        </div>

        <!-- Backup & Restore -->
        <div
          class="rounded-xl p-5"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <h2 class="text-xs font-semibold mb-3" :style="{ color: 'var(--color-text-primary)' }">
            冷备份与恢复
          </h2>
          <div class="space-y-3">
            <div class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">
              导出的文件后缀为 <code>.12words</code>，请保存在 U 盘或私人云盘等安全位置。
            </div>

            <!-- Export -->
            <button
              @click="handleExport"
              class="w-full py-2 rounded-lg text-xs font-medium border cursor-pointer transition-colors duration-100"
              :style="{
                borderColor: 'var(--color-border)',
                color: 'var(--color-text-secondary)',
                background: 'transparent',
              }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
            >
              📤 导出备份（另存为 .12words 文件）
            </button>
            <p v-if="exportMsg" class="text-xs" :style="{ color: exportMsg.includes('成功') ? 'var(--color-success)' : 'var(--color-danger)' }">
              {{ exportMsg }}
            </p>

            <!-- Import -->
            <button
              @click="handleImport"
              :disabled="importing"
              class="w-full py-2 rounded-lg text-xs font-medium border cursor-pointer transition-colors duration-100 disabled:opacity-40"
              :style="{
                borderColor: 'var(--color-border)',
                color: 'var(--color-text-secondary)',
                background: 'transparent',
              }"
              @mouseenter="(e) => { if (!importing) (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
            >
              📥 导入备份（选择 .12words 文件恢复）
            </button>
            <p v-if="importMsg" class="text-xs" :style="{ color: importMsg.includes('成功') ? 'var(--color-success)' : 'var(--color-danger)' }">
              {{ importMsg }}
            </p>
          </div>
        </div>

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
