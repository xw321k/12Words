<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useVaultStore, type VaultEntry } from '../stores/vault'

const vault = useVaultStore()
const searchQuery = ref('')
const showAddDialog = ref(false)
const editingEntry = ref<VaultEntry | null>(null)
const copiedId = ref<string | null>(null)
const copiedField = ref<'username' | 'password' | null>(null)
const toastMsg = ref('')
const toastTimer = ref<ReturnType<typeof setTimeout> | null>(null)

// Form state
const formTitle = ref('')
const formUrl = ref('')
const formUsername = ref('')
const formPassword = ref('')
const formNote = ref('')

const filteredEntries = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return vault.entries
  return vault.entries.filter(e =>
    e.title.toLowerCase().includes(q) ||
    e.username.toLowerCase().includes(q) ||
    e.url.toLowerCase().includes(q)
  )
})

onMounted(async () => {
  await vault.loadEntries()
})

function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer.value) clearTimeout(toastTimer.value)
  toastTimer.value = setTimeout(() => { toastMsg.value = '' }, 2000)
}

function openAdd() {
  formTitle.value = ''
  formUrl.value = ''
  formUsername.value = ''
  formPassword.value = ''
  formNote.value = ''
  editingEntry.value = null
  showAddDialog.value = true
}

function openEdit(entry: VaultEntry) {
  formTitle.value = entry.title
  formUrl.value = entry.url
  formUsername.value = entry.username
  formPassword.value = entry.password
  formNote.value = entry.note
  editingEntry.value = entry
  showAddDialog.value = true
}

async function handleSave() {
  if (!formTitle.value.trim()) return
  if (editingEntry.value) {
    await vault.updateEntry({
      ...editingEntry.value,
      title: formTitle.value.trim(),
      url: formUrl.value.trim(),
      username: formUsername.value.trim(),
      password: formPassword.value,
      note: formNote.value.trim(),
    })
    showToast('已更新')
  } else {
    await vault.addEntry({
      title: formTitle.value.trim(),
      url: formUrl.value.trim(),
      username: formUsername.value.trim(),
      password: formPassword.value,
      note: formNote.value.trim(),
    })
    showToast('已添加')
  }
  showAddDialog.value = false
}

async function handleDelete(id: string) {
  await vault.deleteEntry(id)
  showToast('已删除')
}

async function handleCopy(text: string, id: string, field: 'username' | 'password') {
  await vault.copyToClipboard(text)
  copiedId.value = id
  copiedField.value = field
  showToast('已复制，20秒后自动清除')
  setTimeout(() => {
    if (copiedId.value === id && copiedField.value === field) {
      copiedId.value = null
      copiedField.value = null
    }
  }, 2000)
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}
</script>

<template>
  <div class="flex-1 flex flex-col overflow-hidden" :style="{ background: 'var(--color-surface-secondary)' }">
    <!-- Header -->
    <div
      class="h-14 flex items-center justify-between px-6 border-b flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface)' }"
    >
      <h1 class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">
        密码库
        <span class="ml-2 text-xs font-normal" :style="{ color: 'var(--color-text-tertiary)' }">
          {{ vault.entryCount }} 项
        </span>
      </h1>
      <button
        @click="openAdd"
        class="px-3 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
        :style="{ background: 'var(--color-accent)', color: '#fff' }"
        @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
        @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'"
      >
        + 新增
      </button>
    </div>

    <!-- Search -->
    <div class="px-6 py-3 flex-shrink-0">
      <input
        v-model="searchQuery"
        placeholder="搜索标题、用户名或网址..."
        class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
        :style="{
          background: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          color: 'var(--color-text-primary)',
        }"
      />
    </div>

    <!-- Toast -->
    <div
      v-if="toastMsg"
      class="fixed top-4 left-1/2 -translate-x-1/2 z-50 px-4 py-2 rounded-lg text-xs font-medium shadow-lg"
      :style="{ background: 'var(--color-accent)', color: '#fff' }"
    >
      {{ toastMsg }}
    </div>

    <!-- Empty state -->
    <div
      v-if="vault.entries.length === 0"
      class="flex-1 flex items-center justify-center"
      :style="{ color: 'var(--color-text-tertiary)' }"
    >
      <div class="text-center">
        <div class="text-3xl mb-3">🔐</div>
        <div class="text-sm mb-4">密码库还是空的</div>
        <button
          @click="openAdd"
          class="px-4 py-2 rounded-md text-xs font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
          @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'"
        >
          添加第一个密码
        </button>
      </div>
    </div>

    <!-- Password list -->
    <div v-else class="flex-1 overflow-y-auto px-6 pb-4">
      <div v-if="filteredEntries.length === 0" class="text-center py-12 text-sm" :style="{ color: 'var(--color-text-tertiary)' }">
        没有匹配的结果
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="entry in filteredEntries"
          :key="entry.id"
          class="rounded-lg p-4 transition-shadow duration-100"
          :style="{
            background: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
          }"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <!-- Title -->
              <div class="flex items-center gap-2 mb-1">
                <span class="text-sm font-medium truncate" :style="{ color: 'var(--color-text-primary)' }">
                  {{ entry.title }}
                </span>
                <a
                  v-if="entry.url"
                  :href="entry.url"
                  target="_blank"
                  class="text-xs truncate max-w-[200px] no-underline hover:underline"
                  :style="{ color: 'var(--color-accent)' }"
                >
                  {{ entry.url.replace(/^https?:\/\//, '') }}
                </a>
              </div>

              <!-- Username row -->
              <div
                class="flex items-center gap-2 py-1 px-2 -mx-2 rounded cursor-pointer group"
                :style="{
                  background: copiedId === entry.id && copiedField === 'username'
                    ? 'var(--color-surface-tertiary)' : 'transparent',
                }"
                @click="handleCopy(entry.username, entry.id, 'username')"
              >
                <span class="text-xs w-16 flex-shrink-0" :style="{ color: 'var(--color-text-tertiary)' }">
                  用户名
                </span>
                <span class="text-xs truncate" :style="{ color: 'var(--color-text-primary)' }">
                  {{ entry.username || '(空)' }}
                </span>
                <span
                  class="text-[10px] ml-auto opacity-0 group-hover:opacity-100 transition-opacity"
                  :style="{ color: 'var(--color-accent)' }"
                >
                  点击复制
                </span>
              </div>

              <!-- Password row -->
              <div
                class="flex items-center gap-2 py-1 px-2 -mx-2 rounded cursor-pointer group"
                :style="{
                  background: copiedId === entry.id && copiedField === 'password'
                    ? 'var(--color-surface-tertiary)' : 'transparent',
                }"
                @click="handleCopy(entry.password, entry.id, 'password')"
              >
                <span class="text-xs w-16 flex-shrink-0" :style="{ color: 'var(--color-text-tertiary)' }">
                  密码
                </span>
                <span class="text-xs font-mono select-all" :style="{ color: 'var(--color-text-primary)' }">
                  {'•'.repeat(Math.min(entry.password.length, 20))}
                </span>
                <span
                  class="text-[10px] ml-auto opacity-0 group-hover:opacity-100 transition-opacity"
                  :style="{ color: 'var(--color-accent)' }"
                >
                  点击复制
                </span>
              </div>

              <!-- Note -->
              <div v-if="entry.note" class="mt-1 px-2">
                <span class="text-[11px]" :style="{ color: 'var(--color-text-tertiary)' }">
                  {{ entry.note }}
                </span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-1 ml-3 flex-shrink-0">
              <button
                @click="openEdit(entry)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{
                  borderColor: 'var(--color-border)',
                  color: 'var(--color-text-secondary)',
                  background: 'transparent',
                }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
              >
                编辑
              </button>
              <button
                @click="handleDelete(entry.id)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{
                  borderColor: 'var(--color-border)',
                  color: 'var(--color-danger)',
                  background: 'transparent',
                }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
              >
                删除
              </button>
            </div>
          </div>

          <div class="mt-2 text-[10px]" :style="{ color: 'var(--color-text-tertiary)' }">
            更新于 {{ formatDate(entry.updated_at) }}
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Dialog -->
    <Teleport to="body">
      <div
        v-if="showAddDialog"
        class="fixed inset-0 z-40 flex items-center justify-center"
        :style="{ background: 'rgba(0,0,0,0.4)' }"
        @click.self="showAddDialog = false"
      >
        <div
          class="w-full max-w-md mx-4 rounded-xl p-6"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <h2 class="text-sm font-semibold mb-4" :style="{ color: 'var(--color-text-primary)' }">
            {{ editingEntry ? '编辑密码' : '新增密码' }}
          </h2>

          <div class="space-y-3">
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">标题 *</label>
              <input
                v-model="formTitle"
                placeholder="例如: GitHub"
                class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{
                  background: 'var(--color-surface-secondary)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text-primary)',
                }"
              />
            </div>
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">网址</label>
              <input
                v-model="formUrl"
                placeholder="https://github.com"
                class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{
                  background: 'var(--color-surface-secondary)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text-primary)',
                }"
              />
            </div>
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">用户名</label>
              <input
                v-model="formUsername"
                placeholder="user@example.com"
                class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{
                  background: 'var(--color-surface-secondary)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text-primary)',
                }"
              />
            </div>
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">密码</label>
              <input
                v-model="formPassword"
                type="text"
                placeholder="输入密码或使用生成器"
                class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{
                  background: 'var(--color-surface-secondary)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text-primary)',
                }"
              />
            </div>
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">备注</label>
              <textarea
                v-model="formNote"
                rows="2"
                placeholder="可选备注"
                class="w-full px-3 py-1.5 rounded-md text-xs outline-none resize-none transition-colors duration-100"
                :style="{
                  background: 'var(--color-surface-secondary)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text-primary)',
                }"
              />
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-5">
            <button
              @click="showAddDialog = false"
              class="px-4 py-1.5 rounded-md text-xs border cursor-pointer transition-colors duration-100"
              :style="{
                borderColor: 'var(--color-border)',
                color: 'var(--color-text-secondary)',
                background: 'transparent',
              }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
            >
              取消
            </button>
            <button
              @click="handleSave"
              class="px-4 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
              :style="{ background: 'var(--color-accent)', color: '#fff' }"
              @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
              @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'"
            >
              {{ editingEntry ? '保存' : '添加' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
