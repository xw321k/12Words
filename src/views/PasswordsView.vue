<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVaultStore, type VaultEntry } from '../stores/vault'
import { useLocaleStore } from '../stores/locale'
import { messages, categoryLabels, categoryColors } from '../i18n'

const vault = useVaultStore()
const loc = useLocaleStore()
const searchQuery = ref('')
const showAddDialog = ref(false)
const editingEntry = ref<VaultEntry | null>(null)
const copiedId = ref<string | null>(null)
const copiedField = ref<'username' | 'password' | null>(null)
const toastMsg = ref('')
const toastTimer = ref<ReturnType<typeof setTimeout> | null>(null)
const visiblePwdIds = ref<Set<string>>(new Set())

function t(key: string, params?: Record<string, string | number>): string {
  const msg = messages[loc.locale]?.[key]
  if (!msg) return key
  if (!params) return msg
  return msg.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? ''))
}

function catLabel(cat: string): string {
  return categoryLabels[loc.locale]?.[cat] || cat
}

function catColor(cat: string): string {
  return categoryColors[cat] || '#95A5A6'
}

const categoryTypes = [
  { id: 'Login', icon: '🔑' },
  { id: 'ApiKey', icon: '🔌' },
  { id: 'License', icon: '🏷️' },
  { id: 'Card', icon: '💳' },
  { id: 'Identity', icon: '🆔' },
  { id: 'Crypto', icon: '💰' },
  { id: 'Note', icon: '📄' },
  { id: 'Email', icon: '✉️' },
]

// Form state
const formCategory = ref('Login')
const formTitle = ref('')
const formUrl = ref('')
const formUsername = ref('')
const formPassword = ref('')
const formNote = ref('')
const formFields = ref<Record<string, string>>({})

const categoryFieldDefs: Record<string, { key: string; label: string; type: 'text' | 'textarea' }[]> = {
  Login: [
    { key: 'username', label: '账号', type: 'text' },
    { key: 'password', label: '密码', type: 'text' },
    { key: 'url', label: '网址', type: 'text' },
  ],
  ApiKey: [
    { key: 'api_key', label: 'API Key', type: 'text' },
    { key: 'api_secret', label: 'API Secret', type: 'text' },
    { key: 'endpoint', label: '端点 URL', type: 'text' },
  ],
  License: [
    { key: 'license_key', label: '激活码', type: 'text' },
    { key: 'expiry_date', label: '到期时间', type: 'text' },
  ],
  Card: [
    { key: 'card_number', label: '卡号', type: 'text' },
    { key: 'card_holder', label: '持卡人', type: 'text' },
    { key: 'cvv', label: 'CVV', type: 'text' },
    { key: 'expiry', label: '有效期', type: 'text' },
    { key: 'bank', label: '银行', type: 'text' },
  ],
  Identity: [
    { key: 'full_name', label: '姓名', type: 'text' },
    { key: 'id_number', label: '证件号', type: 'text' },
    { key: 'issue_date', label: '签发日期', type: 'text' },
    { key: 'expiry_date', label: '到期日期', type: 'text' },
    { key: 'country', label: '国家/地区', type: 'text' },
  ],
  Crypto: [
    { key: 'address', label: '钱包地址', type: 'text' },
    { key: 'private_key', label: '私钥', type: 'textarea' },
    { key: 'seed_phrase', label: '助记词', type: 'textarea' },
    { key: 'network', label: '网络', type: 'text' },
  ],
  Note: [
    { key: 'content', label: '内容', type: 'textarea' },
    { key: 'url', label: '网址', type: 'text' },
  ],
  Email: [
    { key: 'email', label: '邮箱账号', type: 'text' },
    { key: 'password', label: '密码', type: 'text' },
    { key: 'url', label: '网址', type: 'text' },
  ],
}

const filteredEntries = computed(() => {
  let list = vault.entries
  if (vault.filterCategory) {
    list = list.filter(e => e.category === vault.filterCategory)
  }
  const q = searchQuery.value.toLowerCase().trim()
  if (q) {
    list = list.filter(e =>
      e.title.toLowerCase().includes(q) ||
      e.username.toLowerCase().includes(q) ||
      e.url.toLowerCase().includes(q)
    )
  }
  return list
})

const activeCategoryLabel = computed(() => {
  if (!vault.filterCategory) return t('sidebar.all')
  return catLabel(vault.filterCategory)
})

onMounted(async () => {
  await vault.loadEntries()
})

function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer.value) clearTimeout(toastTimer.value)
  toastTimer.value = setTimeout(() => { toastMsg.value = '' }, 2000)
}

function isSecretField(label: string): boolean {
  return label === '密码' || label === 'API Key' || label === 'API Secret' || label === '私钥' || label === 'CVV' || label === '助记词'
}

function maskLicenseKey(key: string): string {
  if (key.length <= 8) return '********'
  return key.slice(0, 4) + '********' + key.slice(-4)
}

function openUrl(url: string) {
  let u = url.trim()
  if (!u.startsWith('http://') && !u.startsWith('https://')) u = 'https://' + u
  invoke('open_url', { url: u })
}

function resetForm() {
  formCategory.value = 'Login'
  formTitle.value = ''
  formUrl.value = ''
  formUsername.value = ''
  formPassword.value = ''
  formNote.value = ''
  formFields.value = {}
  editingEntry.value = null
}

function openAdd() {
  resetForm()
  if (vault.filterCategory) formCategory.value = vault.filterCategory
  showAddDialog.value = true
}

function openEdit(entry: VaultEntry) {
  formCategory.value = entry.category
  formTitle.value = entry.title
  formUrl.value = entry.url
  formUsername.value = entry.username
  formPassword.value = entry.password
  formNote.value = entry.note
  try { formFields.value = JSON.parse(entry.fields || '{}') } catch { formFields.value = {} }
  editingEntry.value = entry
  showAddDialog.value = true
}

function buildFields(): string {
  return JSON.stringify(formFields.value)
}

function categoryDisplayVals(entry: VaultEntry): { label: string; value: string; isExpiry?: boolean; daysLeft?: number; isUrl?: boolean; isLicenseKey?: boolean }[] {
  const fields = categoryFieldDefs[entry.category] || []
  const extra: Record<string, string> = {}
  try { Object.assign(extra, JSON.parse(entry.fields || '{}')) } catch {}
  const vals: { label: string; value: string; isExpiry?: boolean; daysLeft?: number; isUrl?: boolean; isLicenseKey?: boolean }[] = []
  if (entry.username && entry.category === 'Login') vals.push({ label: '用户名', value: entry.username })
  if (entry.password && entry.category === 'Login') vals.push({ label: '密码', value: entry.password })
  if (entry.url) vals.push({ label: '网址', value: entry.url })
  for (const f of fields) {
    const v = extra[f.key]
    if (v) {
      const item: any = { label: f.label, value: v }
      if (entry.category === 'License' && f.key === 'expiry_date') {
        item.isExpiry = true
        item.daysLeft = calcDaysLeft(v)
      }
      if (entry.category === 'License' && f.key === 'license_key') {
        item.isLicenseKey = true
      }
      if (f.key === 'url') item.isUrl = true
      vals.push(item)
    }
  }
  return vals
}

function calcDaysLeft(dateStr: string): number {
  const target = new Date(dateStr)
  const now = new Date()
  const diff = target.getTime() - now.getTime()
  return Math.ceil(diff / (1000 * 60 * 60 * 24))
}

function maskPassword(text: string): string {
  if (!text) return ''
  const masked = text.replace(/./g, '•')
  return masked
}

function togglePwd(id: string) {
  const s = new Set(visiblePwdIds.value)
  if (s.has(id)) s.delete(id); else s.add(id)
  visiblePwdIds.value = s
}

async function handleSave() {
  if (!formTitle.value.trim()) return
  const entryData = {
    category: formCategory.value,
    title: formTitle.value.trim(),
    url: formUrl.value.trim(),
    username: formUsername.value.trim(),
    password: formPassword.value,
    note: formNote.value.trim(),
    fields: buildFields(),
  }
  if (editingEntry.value) {
    await vault.updateEntry({ ...editingEntry.value, ...entryData })
    showToast(t('passwords.updated'))
  } else {
    await vault.addEntry(entryData)
    showToast(t('passwords.added'))
  }
  showAddDialog.value = false
}

async function handleDelete(id: string) {
  await vault.deleteEntry(id)
  showToast(t('passwords.deleted'))
}

async function handleCopy(text: string, id: string, field: 'username' | 'password') {
  await vault.copyToClipboard(text)
  copiedId.value = id
  copiedField.value = field
  showToast(t('passwords.copied'))
  setTimeout(() => {
    if (copiedId.value === id && copiedField.value === field) {
      copiedId.value = null
      copiedField.value = null
    }
  }, 2000)
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleDateString(loc.locale === 'zh' ? 'zh-CN' : 'en-US', { year: 'numeric', month: '2-digit', day: '2-digit' })
}
</script>

<template>
  <div class="flex-1 flex flex-col overflow-hidden" :style="{ background: 'var(--color-surface-secondary)' }">
    <div
      class="h-14 flex items-center justify-between px-6 border-b flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface)' }"
    >
      <h1 class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">
        {{ activeCategoryLabel }}
        <span class="ml-2 text-xs font-normal" :style="{ color: 'var(--color-text-tertiary)' }">{{ t('passwords.itemCount', { count: filteredEntries.length }) }}</span>
      </h1>
      <button @click="openAdd"
        class="px-3 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
        :style="{ background: 'var(--color-accent)', color: '#fff' }"
        @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
        @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">
        + {{ t('passwords.add') }}{{ vault.filterCategory ? ' ' + activeCategoryLabel : '' }}
      </button>
    </div>

    <div class="px-6 py-3 flex-shrink-0">
      <input v-model="searchQuery" :placeholder="t('passwords.search')" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
        :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
    </div>

    <div v-if="toastMsg" class="fixed top-4 left-1/2 -translate-x-1/2 z-50 px-4 py-2 rounded-lg text-xs font-medium shadow-lg"
      :style="{ background: 'var(--color-accent)', color: '#fff' }">{{ toastMsg }}</div>

    <div v-if="filteredEntries.length === 0" class="flex-1 flex items-center justify-center" :style="{ color: 'var(--color-text-tertiary)' }">
      <div class="text-center">
        <div class="text-3xl mb-3">🔐</div>
        <div class="text-sm mb-4">{{ vault.entryCount === 0 ? t('passwords.empty') : t('passwords.noMatch') }}</div>
        <button @click="openAdd" class="px-4 py-2 rounded-md text-xs font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
          @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">
          {{ t('passwords.addBtn') }}
        </button>
      </div>
    </div>

    <div v-else class="flex-1 overflow-y-auto px-6 pb-4">
      <div class="space-y-2">
        <div v-for="entry in filteredEntries" :key="entry.id"
          class="rounded-lg p-4 transition-shadow duration-100"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-sm font-medium truncate" :style="{ color: 'var(--color-text-primary)' }">{{ entry.title }}</span>
                <span class="text-[10px] px-1.5 py-0.5 rounded font-medium"
                  :style="{
                    background: catColor(entry.category) + '22',
                    color: catColor(entry.category),
                  }">
                  {{ catLabel(entry.category) }}
                </span>
              </div>
              <div v-for="(v, i) in categoryDisplayVals(entry)" :key="i"
                class="flex items-center gap-2 py-0.5 px-2 -mx-2 rounded group"
                :class="{ 'cursor-pointer': !v.isUrl }"
                @click="v.isUrl ? null : isSecretField(v.label) ? null : handleCopy(v.value, entry.id, 'password')">
                <span class="text-[11px] w-16 flex-shrink-0" :style="{ color: 'var(--color-text-tertiary)' }">{{ v.label }}</span>
                <span v-if="v.isUrl" class="text-xs truncate cursor-pointer underline decoration-dotted underline-offset-2"
                  :style="{ color: 'var(--color-accent)' }"
                  @click.stop="openUrl(v.value)">{{ v.value }}</span>
                <span v-else-if="v.isLicenseKey && !visiblePwdIds.has(entry.id + v.label)" class="text-xs font-mono" :style="{ color: 'var(--color-text-primary)' }">
                  {{ maskLicenseKey(v.value) }}
                </span>
                <span v-else-if="isSecretField(v.label) && !visiblePwdIds.has(entry.id + v.label)" class="text-xs" :style="{ color: 'var(--color-text-primary)' }">
                  {{ maskPassword(v.value) }}
                </span>
                <span v-else class="text-xs truncate" :style="{ color: 'var(--color-text-primary)' }">{{ v.value }}</span>
                <span v-if="v.isExpiry && v.daysLeft !== undefined" class="text-[10px] ml-2 flex-shrink-0 font-medium"
                  :style="{ color: v.daysLeft <= 0 ? 'var(--color-danger)' : v.daysLeft <= 30 ? 'var(--color-warning)' : 'var(--color-success)' }">
                  {{ v.daysLeft <= 0 ? '已过期' : `剩余 ${v.daysLeft} 天` }}
                </span>
                <span v-if="isSecretField(v.label)" class="text-[10px] cursor-pointer transition-colors"
                  :style="{ color: 'var(--color-text-tertiary)' }"
                  @click.stop="togglePwd(entry.id + v.label)">
                  {{ visiblePwdIds.has(entry.id + v.label) ? '🙈' : '👁️' }}
                </span>
                <span class="text-[10px] ml-auto opacity-0 group-hover:opacity-100 transition-opacity" :style="{ color: 'var(--color-accent)' }">{{ t('passwords.copyHint') }}</span>
              </div>
              <div v-if="entry.note" class="mt-1 px-2"><span class="text-[11px]" :style="{ color: 'var(--color-text-tertiary)' }">{{ entry.note }}</span></div>
            </div>
            <div class="flex items-center gap-1 ml-3 flex-shrink-0">
              <button @click="openEdit(entry)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">{{ t('passwords.edit') }}</button>
              <button @click="handleDelete(entry.id)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{ borderColor: 'var(--color-border)', color: 'var(--color-danger)', background: 'transparent' }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">{{ t('passwords.delete') }}</button>
            </div>
          </div>
          <div class="mt-2 text-[10px]" :style="{ color: 'var(--color-text-tertiary)' }">{{ t('passwords.updatedAt', { date: formatDate(entry.updated_at) }) }}</div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Dialog -->
    <Teleport to="body">
      <div v-if="showAddDialog" class="fixed inset-0 z-40 flex items-center justify-center" :style="{ background: 'rgba(0,0,0,0.4)' }">
        <div class="w-full max-w-md mx-4 rounded-xl p-6" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-sm font-semibold mb-4" :style="{ color: 'var(--color-text-primary)' }">
            {{ editingEntry ? t('passwords.editTitle') : t('passwords.addTitle') }} {{ catLabel(formCategory) }}
          </h2>

          <!-- Category selector (only when adding) -->
          <div v-if="!editingEntry" class="mb-4">
            <label class="block text-[11px] mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">{{ t('passwords.category') }}</label>
            <div class="flex flex-wrap gap-1.5">
              <button v-for="cat in categoryTypes" :key="cat.id" @click="formCategory = cat.id"
                class="px-2.5 py-1 rounded text-[11px] border cursor-pointer transition-colors"
                :style="{
                  borderColor: formCategory === cat.id ? catColor(cat.id) : 'var(--color-border)',
                  color: formCategory === cat.id ? catColor(cat.id) : 'var(--color-text-secondary)',
                  background: formCategory === cat.id ? (catColor(cat.id) + '18') : 'transparent',
                }">{{ cat.icon }} {{ catLabel(cat.id) }}</button>
            </div>
          </div>

          <div class="space-y-3">
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t('passwords.titleLabel') }}</label>
              <input v-model="formTitle" :placeholder="t('passwords.titlePlaceholder')" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
            </div>

            <template v-if="formCategory === 'Login'">
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t('field.username') }}</label>
                <input v-model="formUsername" :placeholder="t('passwords.usernamePlaceholder')" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t('field.password') }}</label>
                <input v-model="formPassword" type="text" placeholder="" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t('field.url') }}</label>
                <input v-model="formUrl" placeholder="https://" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
            </template>

            <template v-else>
              <div v-for="fd in categoryFieldDefs[formCategory] || []" :key="fd.key">
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ fd.label }}</label>
                <textarea v-if="fd.type === 'textarea'" v-model="formFields[fd.key]"
                  rows="3" class="w-full px-3 py-1.5 rounded-md text-xs outline-none resize-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
                <input v-else v-model="formFields[fd.key]"
                  :type="fd.key === 'expiry_date' ? 'date' : 'text'"
                  :placeholder="fd.key === 'expiry_date' ? '2026-08-03' : ''"
                  class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
            </template>

            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">{{ t('passwords.noteLabel') }}</label>
              <textarea v-model="formNote" rows="2" :placeholder="t('passwords.notePlaceholder')" class="w-full px-3 py-1.5 rounded-md text-xs outline-none resize-none transition-colors duration-100"
                :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-5">
            <button @click="showAddDialog = false"
              class="px-4 py-1.5 rounded-md text-xs border cursor-pointer transition-colors duration-100"
              :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">{{ t('passwords.cancel') }}</button>
            <button @click="handleSave"
              class="px-4 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
              :style="{ background: 'var(--color-accent)', color: '#fff' }"
              @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
              @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">{{ editingEntry ? t('passwords.save') : t('passwords.addBtn') }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
