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

// Category form config
const categoryTypes = [
  { id: 'Login', label: '账号密码', icon: '🔑' },
  { id: 'ApiKey', label: 'API密钥', icon: '🔌' },
  { id: 'License', label: '软件卡密', icon: '🏷️' },
  { id: 'Card', label: '银行卡', icon: '💳' },
  { id: 'Identity', label: '证件', icon: '🪪' },
  { id: 'Crypto', label: '钱包私钥', icon: '₿' },
  { id: 'Note', label: '加密便签', icon: '📄' },
]

// Form state
const formCategory = ref('Login')
const formTitle = ref('')
const formUrl = ref('')
const formUsername = ref('')
const formPassword = ref('')
const formNote = ref('')
const formFields = ref<Record<string, string>>({})

// Extra field schemas per category
const categoryFieldDefs: Record<string, { key: string; label: string; type: 'text' | 'textarea' }[]> = {
  Login: [
    { key: 'username', label: '用户名', type: 'text' },
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
    { key: 'product', label: '产品名称', type: 'text' },
    { key: 'purchase_date', label: '购买日期', type: 'text' },
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
  ],
}

const filteredEntries = computed(() => {
  let list = vault.entries
  // Filter by category
  if (vault.filterCategory) {
    list = list.filter(e => e.category === vault.filterCategory)
  }
  // Filter by search
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
  if (!vault.filterCategory) return '全部'
  return categoryTypes.find(c => c.id === vault.filterCategory)?.label || vault.filterCategory
})

onMounted(async () => {
  await vault.loadEntries()
})

function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer.value) clearTimeout(toastTimer.value)
  toastTimer.value = setTimeout(() => { toastMsg.value = '' }, 2000)
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

function categoryDisplayVals(entry: VaultEntry): { label: string; value: string }[] {
  const fields = categoryFieldDefs[entry.category] || []
  const extra: Record<string, string> = {}
  try { Object.assign(extra, JSON.parse(entry.fields || '{}')) } catch {}
  const vals: { label: string; value: string }[] = []
  if (entry.username && entry.category === 'Login') vals.push({ label: '用户名', value: entry.username })
  if (entry.password && entry.category === 'Login') vals.push({ label: '密码', value: entry.password })
  if (entry.url) vals.push({ label: '网址', value: entry.url })
  for (const f of fields) {
    const v = extra[f.key]
    if (v) vals.push({ label: f.label, value: v })
  }
  return vals
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
    showToast('已更新')
  } else {
    await vault.addEntry(entryData)
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
    <div
      class="h-14 flex items-center justify-between px-6 border-b flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface)' }"
    >
      <h1 class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">
        {{ activeCategoryLabel }}
        <span class="ml-2 text-xs font-normal" :style="{ color: 'var(--color-text-tertiary)' }">{{ filteredEntries.length }} 项</span>
      </h1>
      <button @click="openAdd"
        class="px-3 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
        :style="{ background: 'var(--color-accent)', color: '#fff' }"
        @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
        @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">
        + 新增{{ vault.filterCategory ? ' ' + activeCategoryLabel : '' }}
      </button>
    </div>

    <div class="px-6 py-3 flex-shrink-0">
      <input v-model="searchQuery" placeholder="搜索..." class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
        :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
    </div>

    <div v-if="toastMsg" class="fixed top-4 left-1/2 -translate-x-1/2 z-50 px-4 py-2 rounded-lg text-xs font-medium shadow-lg"
      :style="{ background: 'var(--color-accent)', color: '#fff' }">{{ toastMsg }}</div>

    <div v-if="filteredEntries.length === 0" class="flex-1 flex items-center justify-center" :style="{ color: 'var(--color-text-tertiary)' }">
      <div class="text-center">
        <div class="text-3xl mb-3">🔐</div>
        <div class="text-sm mb-4">{{ vault.entryCount === 0 ? '密码库还是空的' : '没有匹配的结果' }}</div>
        <button @click="openAdd" class="px-4 py-2 rounded-md text-xs font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
          @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">
          添加
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
                <span class="text-[10px] px-1.5 py-0.5 rounded" :style="{ background: 'var(--color-surface-tertiary)', color: 'var(--color-text-tertiary)' }">
                  {{ categoryTypes.find(c => c.id === entry.category)?.label || entry.category }}
                </span>
              </div>
              <div v-for="(v, i) in categoryDisplayVals(entry)" :key="i"
                class="flex items-center gap-2 py-0.5 px-2 -mx-2 rounded cursor-pointer group"
                @click="handleCopy(v.value, entry.id, 'password')">
                <span class="text-[11px] w-16 flex-shrink-0" :style="{ color: 'var(--color-text-tertiary)' }">{{ v.label }}</span>
                <span class="text-xs truncate" :style="{ color: 'var(--color-text-primary)' }">{{ v.value }}</span>
                <span class="text-[10px] ml-auto opacity-0 group-hover:opacity-100 transition-opacity" :style="{ color: 'var(--color-accent)' }">点击复制</span>
              </div>
              <div v-if="entry.note" class="mt-1 px-2"><span class="text-[11px]" :style="{ color: 'var(--color-text-tertiary)' }">{{ entry.note }}</span></div>
            </div>
            <div class="flex items-center gap-1 ml-3 flex-shrink-0">
              <button @click="openEdit(entry)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">编辑</button>
              <button @click="handleDelete(entry.id)"
                class="px-2 py-1 rounded text-[11px] border cursor-pointer transition-colors duration-100"
                :style="{ borderColor: 'var(--color-border)', color: 'var(--color-danger)', background: 'transparent' }"
                @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
                @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">删除</button>
            </div>
          </div>
          <div class="mt-2 text-[10px]" :style="{ color: 'var(--color-text-tertiary)' }">更新于 {{ formatDate(entry.updated_at) }}</div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Dialog -->
    <Teleport to="body">
      <div v-if="showAddDialog" class="fixed inset-0 z-40 flex items-center justify-center" :style="{ background: 'rgba(0,0,0,0.4)' }" @click.self="showAddDialog = false">
        <div class="w-full max-w-md mx-4 rounded-xl p-6" :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }">
          <h2 class="text-sm font-semibold mb-4" :style="{ color: 'var(--color-text-primary)' }">
            {{ editingEntry ? '编辑' : '新增' }} {{ categoryTypes.find(c => c.id === formCategory)?.label || '' }}
          </h2>

          <!-- Category selector (only when adding) -->
          <div v-if="!editingEntry" class="mb-4">
            <label class="block text-[11px] mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">分类</label>
            <div class="flex flex-wrap gap-1.5">
              <button v-for="cat in categoryTypes" :key="cat.id" @click="formCategory = cat.id"
                class="px-2.5 py-1 rounded text-[11px] border cursor-pointer transition-colors"
                :style="{
                  borderColor: formCategory === cat.id ? 'var(--color-accent)' : 'var(--color-border)',
                  color: formCategory === cat.id ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                  background: formCategory === cat.id ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'transparent',
                }">{{ cat.icon }} {{ cat.label }}</button>
            </div>
          </div>

          <div class="space-y-3">
            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">标题 *</label>
              <input v-model="formTitle" placeholder="例如: GitHub" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
            </div>

            <!-- Dynamic fields based on category -->
            <template v-if="formCategory === 'Login'">
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">用户名</label>
                <input v-model="formUsername" placeholder="user@example.com" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">密码</label>
                <input v-model="formPassword" type="text" placeholder="密码" class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
              <div>
                <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">网址</label>
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
                  class="w-full px-3 py-1.5 rounded-md text-xs outline-none transition-colors duration-100"
                  :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
              </div>
            </template>

            <div>
              <label class="block text-[11px] mb-1" :style="{ color: 'var(--color-text-secondary)' }">备注</label>
              <textarea v-model="formNote" rows="2" placeholder="可选备注" class="w-full px-3 py-1.5 rounded-md text-xs outline-none resize-none transition-colors duration-100"
                :style="{ background: 'var(--color-surface-secondary)', border: '1px solid var(--color-border)', color: 'var(--color-text-primary)' }" />
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-5">
            <button @click="showAddDialog = false"
              class="px-4 py-1.5 rounded-md text-xs border cursor-pointer transition-colors duration-100"
              :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
              @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
              @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }">取消</button>
            <button @click="handleSave"
              class="px-4 py-1.5 rounded-md text-xs font-medium border-none cursor-pointer transition-colors duration-100"
              :style="{ background: 'var(--color-accent)', color: '#fff' }"
              @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
              @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'">{{ editingEntry ? '保存' : '添加' }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
