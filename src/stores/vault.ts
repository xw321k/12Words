import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { save, open } from '@tauri-apps/plugin-dialog'

export interface MnemonicResult {
  phrase: string
  seed_hex: string
  user_id: string
}

export interface VaultEntry {
  id: string
  category: string
  title: string
  url: string
  username: string
  password: string
  note: string
  fields: string
  updated_at: string
}

function makeId(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8)
}

function nowISO(): string {
  return new Date().toISOString()
}

export const useVaultStore = defineStore('vault', () => {
  // -- State --
  const mnemonic = ref<MnemonicResult | null>(null)
  const isInitialized = ref(false)
  const entries = ref<VaultEntry[]>([])
  const clipboardTimer = ref<ReturnType<typeof setTimeout> | null>(null)
  const filterCategory = ref('')
  const isFromStorage = ref(false)

  // -- Computed --
  const phrase = computed(() => mnemonic.value?.phrase ?? '')
  const words = computed(() => phrase.value.split(' '))
  const seedHex = computed(() => mnemonic.value?.seed_hex ?? '')
  const userId = computed(() => mnemonic.value?.user_id ?? '')
  const entryCount = computed(() => entries.value.length)

  // -- Clipboard --
  async function copyToClipboard(text: string) {
    await writeText(text)
    // Clear after 20 seconds
    if (clipboardTimer.value) clearTimeout(clipboardTimer.value)
    clipboardTimer.value = setTimeout(async () => {
      try {
        await writeText('')
      } catch { /* ignore */ }
    }, 20000)
  }

  // -- Actions --
  async function initialize() {
    const saved = localStorage.getItem('vault_mnemonic')
    if (saved) {
      try {
        const parsed = JSON.parse(saved) as MnemonicResult
        mnemonic.value = parsed
        isInitialized.value = true
        isFromStorage.value = true
        return
      } catch {
        localStorage.removeItem('vault_mnemonic')
      }
    }
    isInitialized.value = false
  }

  async function generate() {
    const result = await invoke<MnemonicResult>('generate_mnemonic')
    mnemonic.value = result
    // NOT persisted yet — wait for confirmation
    return result
  }

  function confirmMnemonic() {
    if (!mnemonic.value) return
    localStorage.setItem('vault_mnemonic', JSON.stringify(mnemonic.value))
    isInitialized.value = true
  }

  async function importPhrase(phrase: string) {
    const result = await invoke<MnemonicResult>('import_mnemonic', { phrase })
    mnemonic.value = result
    localStorage.setItem('vault_mnemonic', JSON.stringify(result))
    isInitialized.value = true
    return result
  }

  function lock() {
    entries.value = []
    mnemonic.value = null
    isInitialized.value = false
    localStorage.removeItem('vault_mnemonic')
  }

  async function loadEntries() {
    if (!seedHex.value) return
    entries.value = await invoke<VaultEntry[]>('read_vault', { seedHex: seedHex.value })
  }

  async function saveEntries() {
    if (!seedHex.value) return
    await invoke('write_vault', { seedHex: seedHex.value, entries: entries.value })
  }

  async function addEntry(entry: Omit<VaultEntry, 'id' | 'updated_at'>) {
    const newEntry: VaultEntry = {
      ...entry,
      id: makeId(),
      updated_at: nowISO(),
    }
    entries.value = [...entries.value, newEntry]
    await saveEntries()
  }

  async function updateEntry(entry: VaultEntry) {
    entries.value = entries.value.map(e =>
      e.id === entry.id ? { ...entry, updated_at: nowISO() } : e
    )
    await saveEntries()
  }

  async function deleteEntry(id: string) {
    entries.value = entries.value.filter(e => e.id !== id)
    await saveEntries()
  }

  async function generatePassword(length: number, numbers: boolean, symbols: boolean): Promise<string> {
    return await invoke<string>('generate_password', { length, useNumbers: numbers, useSymbols: symbols })
  }

  async function exportBackup(): Promise<string> {
    // Read raw encrypted data
    const raw = await invoke<number[]>('read_vault_raw')
    const data = new Uint8Array(raw)
    // Ask where to save
    const dest = await save({
      filters: [{ name: '加密密码库', extensions: ['12words'] }],
      defaultPath: 'vault_export.12words',
    })
    if (!dest) throw '用户取消了导出'
    return await invoke<string>('export_vault', { destPath: dest, encryptedData: Array.from(data) })
  }

  async function importBackup(): Promise<VaultEntry[]> {
    const file = await open({
      filters: [{ name: '加密密码库', extensions: ['12words'] }],
      multiple: false,
    })
    if (!file) throw '用户取消了导入'
    const result = await invoke<VaultEntry[]>('import_vault_from', {
      sourcePath: file,
      seedHex: seedHex.value,
    })
    entries.value = result
    return result
  }

  async function saveMnemonicTxt(): Promise<string> {
    return await invoke<string>('save_mnemonic_txt', { phrase: phrase.value })
  }

  async function vaultExists(): Promise<boolean> {
    return await invoke<boolean>('vault_exists')
  }

  async function openUrl(url: string) {
    await invoke('open_url', { url })
  }

  return {
    mnemonic, isInitialized, entries, clipboardTimer,
    phrase, words, seedHex, userId, entryCount, filterCategory, isFromStorage,
    initialize, generate, confirmMnemonic, importPhrase, lock,
    loadEntries, saveEntries, addEntry, updateEntry, deleteEntry,
    generatePassword, copyToClipboard, exportBackup, importBackup, saveMnemonicTxt, vaultExists, openUrl,
  }
})
