import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface MnemonicResult {
  phrase: string
  seed_hex: string
  user_id: string
}

export interface VaultEntry {
  id: string
  title: string
  url: string
  username: string
  password: string
  note: string
  updated_at: string
}

export const useVaultStore = defineStore('vault', () => {
  // -- State --
  const mnemonic = ref<MnemonicResult | null>(null)
  const isInitialized = ref(false)

  // -- Computed --
  const phrase = computed(() => mnemonic.value?.phrase ?? '')
  const words = computed(() => phrase.value.split(' '))
  const seedHex = computed(() => mnemonic.value?.seed_hex ?? '')
  const userId = computed(() => mnemonic.value?.user_id ?? '')

  // -- Actions --
  async function initialize() {
    // Try to load existing mnemonic from localStorage
    const saved = localStorage.getItem('vault_mnemonic')
    if (saved) {
      try {
        const parsed = JSON.parse(saved) as MnemonicResult
        mnemonic.value = parsed
        isInitialized.value = true
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
    localStorage.setItem('vault_mnemonic', JSON.stringify(result))
    isInitialized.value = true
    return result
  }

  async function importPhrase(phrase: string) {
    const result = await invoke<MnemonicResult>('import_mnemonic', { phrase })
    mnemonic.value = result
    localStorage.setItem('vault_mnemonic', JSON.stringify(result))
    isInitialized.value = true
    return result
  }

  function lock() {
    mnemonic.value = null
    isInitialized.value = false
    localStorage.removeItem('vault_mnemonic')
  }

  return {
    mnemonic,
    isInitialized,
    phrase,
    words,
    seedHex,
    userId,
    initialize,
    generate,
    importPhrase,
    lock,
  }
})
