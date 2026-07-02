<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVaultStore } from '../stores/vault'
import { useLocaleStore } from '../stores/locale'
import { messages } from '../i18n'

const router = useRouter()
const vault = useVaultStore()
const loc = useLocaleStore()

type Step = 'generate' | 'show' | 'verify' | 'done'
const step = ref<Step>('generate')
const loading = ref(false)
const error = ref('')
const seedInfo = ref('')

// Import mode
const showImportInput = ref(false)
const wordInputs = ref<string[]>(Array(12).fill(''))

// Verification
const verifyIndex = ref(0)
const currentOptions = ref<string[]>([])
const userChoices = ref<string[]>([])
const verifyError = ref(false)
const isImport = ref(false)
const savedTxtPath = ref('')

function t(key: string, params?: Record<string, string | number>): string {
  const msg = messages[loc.locale]?.[key]
  if (!msg) return key
  if (!params) return msg
  return msg.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? ''))
}

function shuffle(arr: string[]): string[] {
  const a = [...arr]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

function pickN(arr: string[], n: number): string[] {
  const shuffled = shuffle(arr)
  return shuffled.slice(0, n)
}

function generateOptions(correctWord: string): string[] {
  const alreadyUsed = userChoices.value.filter(w => w !== '')
  const pool = vault.words.filter(w => w !== correctWord && !alreadyUsed.includes(w))
  let distractors = pickN(pool, 4)
  const fallback = ['abandon', 'ability', 'able', 'about', 'above', 'absent',
    'absorb', 'abstract', 'absurd', 'abuse', 'access', 'accident', 'account',
    'accuse', 'achieve', 'acid', 'acoustic', 'acquire', 'across', 'act',
    'action', 'actor', 'actress', 'actual', 'adapt', 'add', 'addict',
    'address', 'adjust', 'admit', 'adult', 'advance', 'advice', 'aerobic',
    'affair', 'afford', 'afraid', 'again', 'age', 'agent', 'agree',
    'ahead', 'aim', 'air', 'airport', 'aisle', 'alarm', 'album']
  while (distractors.length < 4) {
    const w = fallback[Math.floor(Math.random() * fallback.length)]
    if (!distractors.includes(w) && w !== correctWord && !alreadyUsed.includes(w)) {
      distractors = [...distractors, w]
    }
  }
  return shuffle([correctWord, ...distractors.slice(0, 4)])
}

async function handleGenerate() {
  loading.value = true
  error.value = ''
  try {
    await vault.generate()
    seedInfo.value = `Seed: ${vault.userId.slice(0, 16)}...`
    isImport.value = false
    step.value = 'show'
  } catch (e: any) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function handleSaveTxt() {
  try {
    const path = await vault.saveMnemonicTxt()
    savedTxtPath.value = path
  } catch (e: any) {
    error.value = String(e)
  }
}

function showImport() {
  error.value = ''
  wordInputs.value = ['']
  showImportInput.value = true
}

function onImportPaste(e: Event) {
  const val = (e.target as HTMLTextAreaElement).value
  wordInputs.value[0] = val
}

async function handleImport() {
  const text = wordInputs.value[0].trim().toLowerCase()
  const words = text.split(/\s+/).filter(Boolean)
  if (words.length !== 12) {
    error.value = `请提供 12 个助记词（当前 ${words.length} 个）`
    return
  }
  const phrase = words.join(' ')
  loading.value = true
  error.value = ''
  try {
    await vault.importPhrase(phrase)
    const exists = await vault.vaultExists()
    if (!exists) {
      try {
        await vault.importBackup()
      } catch (e: any) {
        loading.value = false
        if (e === '用户取消了导入') return
        error.value = '无法解密备份文件，请确认助记词和备份文件匹配'
        return
      }
    } else {
      await vault.loadEntries()
    }
    isImport.value = true
    showImportInput.value = false
    step.value = 'done'
  } catch (e: any) {
    error.value = '无效的助记词，请检查拼写'
  } finally {
    loading.value = false
  }
}

function handleShowDone() {
  userChoices.value = new Array(12).fill('')
  verifyIndex.value = 0
  verifyError.value = false
  currentOptions.value = generateOptions(vault.words[0])
  step.value = 'verify'
}

function handleWordClick(word: string) {
  if (verifyIndex.value >= 12) return
  userChoices.value[verifyIndex.value] = word
  verifyIndex.value++
  if (verifyIndex.value < 12) {
    currentOptions.value = generateOptions(vault.words[verifyIndex.value])
  }
}

function handleSubmitVerify() {
  const allCorrect = vault.words.every((w, i) => w === userChoices.value[i])
  if (allCorrect) {
    step.value = 'done'
  } else {
    verifyError.value = true
    userChoices.value = new Array(12).fill('')
    verifyIndex.value = 0
    currentOptions.value = generateOptions(vault.words[0])
  }
}

function handleDone() {
  vault.confirmMnemonic()
  router.push('/passwords')
}

function backToStart() {
  step.value = 'generate'
  showImportInput.value = false
  isImport.value = false
  wordInputs.value = Array(12).fill('')
  savedTxtPath.value = ''
  error.value = ''
}
</script>

<template>
  <div
    class="h-screen w-screen flex items-center justify-center"
    :style="{ background: 'var(--color-surface-secondary)' }"
  >
    <!-- Language switcher -->
    <div class="fixed top-4 right-4 flex gap-1.5 z-10">
      <button @click="loc.setLocale('zh')"
        class="px-2 py-1 rounded text-[10px] font-medium border cursor-pointer transition-colors"
        :style="{
          borderColor: loc.locale === 'zh' ? 'var(--color-accent)' : 'var(--color-border)',
          color: loc.locale === 'zh' ? 'var(--color-accent)' : 'var(--color-text-tertiary)',
          background: loc.locale === 'zh' ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'transparent',
        }">中文</button>
      <button @click="loc.setLocale('en')"
        class="px-2 py-1 rounded text-[10px] font-medium border cursor-pointer transition-colors"
        :style="{
          borderColor: loc.locale === 'en' ? 'var(--color-accent)' : 'var(--color-border)',
          color: loc.locale === 'en' ? 'var(--color-accent)' : 'var(--color-text-tertiary)',
          background: loc.locale === 'en' ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'transparent',
        }">EN</button>
    </div>
    <div class="w-full max-w-lg mx-6">
      <!-- ========== Main Page ========== -->
      <div v-if="step === 'generate'" class="text-center">
        <div class="text-4xl mb-4">🔐</div>
        <h1 class="text-xl font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          {{ t('onboarding.welcome') }}
        </h1>
        <p class="text-sm mb-8" :style="{ color: 'var(--color-text-secondary)' }"
          v-html="t('onboarding.desc')" />
        <div v-if="error" class="mb-4 text-xs" :style="{ color: 'var(--color-danger)' }">{{ error }}</div>

        <button
          @click="handleGenerate"
          :disabled="loading"
          class="w-full py-2.5 rounded-lg text-sm font-medium transition-colors duration-100 border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff', opacity: loading ? 0.6 : 1 }"
          @mouseenter="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          {{ loading ? t('onboarding.processing') : t('onboarding.generate') }}
        </button>

        <div class="flex items-center gap-3 my-5">
          <div class="flex-1 h-px" :style="{ background: 'var(--color-border)' }" />
          <span class="text-xs" :style="{ color: 'var(--color-text-tertiary)' }">{{ t('onboarding.or') }}</span>
          <div class="flex-1 h-px" :style="{ background: 'var(--color-border)' }" />
        </div>

        <div v-if="!showImportInput">
          <button
            @click="showImport"
            :disabled="loading"
            class="w-full py-2.5 rounded-lg text-sm font-medium border cursor-pointer transition-colors duration-100"
            :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
            @mouseenter="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
            @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
          >
            {{ t('onboarding.import') }}
          </button>

          <!-- 12Words 提示 -->
          <div
            class="mt-6 rounded-xl p-4 text-left"
            :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-accent)' }"
          >
            <div class="flex items-start gap-2">
              <span class="text-sm flex-shrink-0 mt-0.5">🛡️</span>
              <div class="text-[11px] space-y-1.5" :style="{ color: 'var(--color-text-secondary)' }">
                <p class="font-semibold" :style="{ color: 'var(--color-text-primary)' }">{{ t('onboarding.tipTitle') }}</p>
                <p>{{ t('onboarding.tipLine1') }}</p>
                <p class="font-medium" :style="{ color: 'var(--color-text-primary)' }">{{ t('onboarding.tipLine2') }}</p>
                <p>{{ t('onboarding.tipLine3') }}</p>
                <ul class="list-disc pl-4">
                  <li>{{ t('onboarding.tipItem1') }}</li>
                  <li>{{ t('onboarding.tipItem2') }}</li>
                </ul>
              </div>
            </div>
          </div>
        </div>

        <!-- Import: textarea -->
        <div v-else class="text-left">
          <p class="text-xs mb-3 text-center" :style="{ color: 'var(--color-text-secondary)' }">
            {{ t('onboarding.importTitle') }}
          </p>
          <textarea
            v-model="wordInputs[0]"
            @input="onImportPaste"
            :placeholder="t('onboarding.importPlaceholder')"
            rows="4"
            class="w-full px-3 py-2 rounded-lg text-xs outline-none resize-none transition-colors duration-100"
            :style="{
              background: 'var(--color-surface)',
              border: '1px solid var(--color-border)',
              color: 'var(--color-text-primary)',
            }"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
          />
          <div class="flex gap-2 mt-3">
            <button
              @click="showImportInput = false"
              class="flex-1 py-2 rounded-lg text-xs border cursor-pointer"
              :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-tertiary)', background: 'transparent' }"
            >
              {{ t('onboarding.cancel') }}
            </button>
            <button
              @click="handleImport"
              :disabled="loading || !wordInputs[0].trim()"
              class="flex-1 py-2 rounded-lg text-xs font-medium border-none cursor-pointer disabled:opacity-40"
              :style="{ background: 'var(--color-accent)', color: '#fff' }"
            >
              {{ loading ? t('onboarding.verifying') : t('onboarding.importBtn') }}
            </button>
          </div>
        </div>
      </div>

      <!-- ========== Show (backup) ========== -->
      <div v-else-if="step === 'show'" class="text-center">
        <button
          @click="backToStart"
          class="flex items-center gap-1 text-xs border-none cursor-pointer bg-transparent mb-4"
          :style="{ color: 'var(--color-text-tertiary)' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-secondary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-tertiary)' }"
        >
          ← {{ t('onboarding.back') }}
        </button>
        <div class="text-4xl mb-4">📝</div>
        <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          {{ t('onboarding.backupTitle') }}
        </h2>
        <p class="text-xs mb-5" :style="{ color: 'var(--color-text-secondary)' }"
          v-html="t('onboarding.backupDesc')" />
        <div
          class="rounded-lg p-4 mb-6 text-left"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <div class="flex flex-wrap gap-2">
            <span
              v-for="(word, i) in vault.words"
              :key="i"
              class="px-3 py-1.5 rounded-md text-sm font-medium"
              :style="{ background: 'var(--color-surface-tertiary)', color: 'var(--color-text-primary)' }"
            >
              {{ word }}
            </span>
          </div>
          <div class="mt-3 text-xs text-center" :style="{ color: 'var(--color-text-tertiary)' }">
            {{ seedInfo }}
          </div>
        </div>

        <button
          @click="handleSaveTxt"
          class="w-full py-2 rounded-lg text-xs font-medium border cursor-pointer transition-colors duration-100 mb-3"
          :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
        >
          {{ t('onboarding.saveTxt') }}
        </button>
        <p v-if="savedTxtPath" class="text-xs mb-3" :style="{ color: 'var(--color-success)' }">
          {{ t('onboarding.savedTxt', { path: savedTxtPath }) }}
        </p>

        <button
          @click="handleShowDone"
          class="w-full py-2.5 rounded-lg text-sm font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          {{ t('onboarding.startVerify') }}
        </button>
      </div>

      <!-- ========== Verify ========== -->
      <div v-else-if="step === 'verify'">
        <button
          @click="isImport ? backToStart() : (step = 'show')"
          class="flex items-center gap-1 text-xs border-none cursor-pointer bg-transparent mb-4"
          :style="{ color: 'var(--color-text-tertiary)' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-secondary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-tertiary)' }"
        >
          ← {{ t('onboarding.back') }}
        </button>

        <h2 class="text-lg font-semibold mb-1 text-center" :style="{ color: 'var(--color-text-primary)' }">
          {{ t('onboarding.verifyTitle') }}
        </h2>
        <p class="text-xs mb-4 text-center" :style="{ color: 'var(--color-text-secondary)' }"
          v-html="t('onboarding.verifyStep', { index: verifyIndex + 1 })" />

        <div class="h-1 rounded-full mb-4" :style="{ background: 'var(--color-surface-tertiary)' }">
          <div
            class="h-full rounded-full transition-all duration-200"
            :style="{ width: `${(verifyIndex / 12) * 100}%`, background: 'var(--color-accent)' }"
          />
        </div>

        <div
          class="flex flex-wrap gap-1.5 mb-4 p-3 rounded-lg min-h-[44px]"
          :style="{
            background: 'var(--color-surface)',
            border: verifyError ? '1px solid var(--color-danger)' : '1px solid var(--color-border)',
          }"
        >
          <span
            v-for="(choice, i) in userChoices"
            :key="i"
            class="px-2 py-0.5 rounded text-xs font-medium"
            :style="{
              background: choice ? 'var(--color-accent)' : 'var(--color-surface-tertiary)',
              color: choice ? '#fff' : 'var(--color-text-tertiary)',
            }"
          >
            {{ i + 1 }}.{{ choice || '___' }}
          </span>
        </div>

        <div v-if="verifyIndex < 12" class="flex flex-wrap gap-3 justify-center mb-4">
          <button
            v-for="word in currentOptions"
            :key="word"
            @click="handleWordClick(word)"
            class="px-5 py-3 rounded-lg text-sm font-medium border-2 cursor-pointer transition-all duration-150"
            :style="{
              background: 'var(--color-surface)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text-primary)',
              minWidth: '100px',
            }"
            @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-accent)' }"
            @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)' }"
          >
            {{ word }}
          </button>
        </div>

        <div v-else class="text-center">
          <button
            @click="handleSubmitVerify"
            class="w-full py-2.5 rounded-lg text-sm font-medium border-none cursor-pointer mb-3"
            :style="{ background: 'var(--color-accent)', color: '#fff' }"
            @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
            @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
          >
            {{ t('onboarding.verifySubmit') }}
          </button>
        </div>

        <p v-if="verifyError" class="text-xs text-center" :style="{ color: 'var(--color-danger)' }">
          {{ t('onboarding.verifyError') }}
        </p>
      </div>

      <!-- ========== Done ========== -->
      <div v-else-if="step === 'done'" class="text-center">
        <div class="text-4xl mb-4">✅</div>
        <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          {{ t('onboarding.verifiedTitle') }}
        </h2>
        <p class="text-xs mb-6" :style="{ color: 'var(--color-text-secondary)' }"
          v-html="t('onboarding.verifiedDesc')" />
        <div
          class="rounded-lg p-4 mb-6 text-left"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <div class="text-xs font-semibold mb-2" :style="{ color: 'var(--color-text-secondary)' }">
            {{ t('onboarding.wordsTitle') }}
          </div>
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="(word, i) in vault.words"
              :key="i"
              class="px-2 py-0.5 rounded text-xs font-medium"
              :style="{ background: 'var(--color-surface-tertiary)', color: 'var(--color-text-primary)' }"
            >
              {{ word }}
            </span>
          </div>
          <div class="mt-3 text-xs" :style="{ color: 'var(--color-text-tertiary)' }">{{ seedInfo }}</div>
        </div>
        <button
          @click="handleDone"
          class="w-full py-2.5 rounded-lg text-sm font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          {{ t('onboarding.enterVault') }}
        </button>
      </div>
    </div>
  </div>
</template>
