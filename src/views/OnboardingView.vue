<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVaultStore } from '../stores/vault'

const router = useRouter()
const vault = useVaultStore()

type Step = 'generate' | 'show' | 'verify' | 'done'
const step = ref<Step>('generate')
const loading = ref(false)
const error = ref('')
const seedInfo = ref('')

// Import mode
const importPhrase = ref('')
const showImportInput = ref(false)

// Verification
const verifyIndex = ref(0)
const currentOptions = ref<string[]>([])
const userChoices = ref<string[]>([])
const verifyError = ref(false)
const isImport = ref(false)

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

function showImport() {
  showImportInput.value = true
  error.value = ''
}

async function handleImport() {
  const phrase = importPhrase.value.trim().toLowerCase().replace(/\s+/g, ' ')
  const words = phrase.split(' ')
  if (words.length !== 12) {
    error.value = '请输入 12 个助记词，用空格分隔'
    return
  }
  loading.value = true
  error.value = ''
  try {
    await vault.importPhrase(phrase)
    seedInfo.value = `Seed: ${vault.userId.slice(0, 16)}...`
    isImport.value = true
    showImportInput.value = false
    // Skip show step, go straight to verify
    userChoices.value = new Array(12).fill('')
    verifyIndex.value = 0
    verifyError.value = false
    currentOptions.value = generateOptions(vault.words[0])
    step.value = 'verify'
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
  importPhrase.value = ''
  showImportInput.value = false
  isImport.value = false
  error.value = ''
}
</script>

<template>
  <div
    class="h-screen w-screen flex items-center justify-center"
    :style="{ background: 'var(--color-surface-secondary)' }"
  >
    <div class="w-full max-w-lg mx-6">
      <!-- ========== Step: Generate / Import ========== -->
      <div v-if="step === 'generate'" class="text-center">
        <div class="text-4xl mb-4">🔐</div>
        <h1 class="text-xl font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          欢迎使用 12Words
        </h1>
        <p class="text-sm mb-8" :style="{ color: 'var(--color-text-secondary)' }">
          12 位助记词是你访问密码库的<strong>唯一钥匙</strong>
        </p>
        <div v-if="error" class="mb-4 text-xs" :style="{ color: 'var(--color-danger)' }">{{ error }}</div>

        <!-- Generate new -->
        <button
          @click="handleGenerate"
          :disabled="loading"
          class="w-full py-2.5 rounded-lg text-sm font-medium transition-colors duration-100 border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff', opacity: loading ? 0.6 : 1 }"
          @mouseenter="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          {{ loading ? '处理中...' : '生成新助记词' }}
        </button>

        <div class="flex items-center gap-3 my-5">
          <div class="flex-1 h-px" :style="{ background: 'var(--color-border)' }" />
          <span class="text-xs" :style="{ color: 'var(--color-text-tertiary)' }">或者</span>
          <div class="flex-1 h-px" :style="{ background: 'var(--color-border)' }" />
        </div>

        <!-- Verify existing mnemonic -->
        <button
          v-if="!showImportInput"
          @click="showImport"
          :disabled="loading"
          class="w-full py-2.5 rounded-lg text-sm font-medium border cursor-pointer transition-colors duration-100"
          :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)', background: 'transparent' }"
          @mouseenter="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
        >
          验证已有助记词
        </button>

        <!-- Import textarea (appears after clicking verify existing) -->
        <div v-else class="text-left">
          <textarea
            v-model="importPhrase"
            placeholder="在此粘贴 12 个助记词，用空格分隔..."
            rows="3"
            class="w-full px-3 py-2 rounded-lg text-xs outline-none resize-none transition-colors duration-100 mb-3"
            :style="{
              background: 'var(--color-surface)',
              border: '1px solid var(--color-border)',
              color: 'var(--color-text-primary)',
            }"
          />
          <div class="flex gap-2">
            <button
              @click="showImportInput = false"
              class="flex-1 py-2 rounded-lg text-xs border cursor-pointer"
              :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-tertiary)', background: 'transparent' }"
            >
              取消
            </button>
            <button
              @click="handleImport"
              :disabled="loading || !importPhrase.trim()"
              class="flex-1 py-2 rounded-lg text-xs font-medium border-none cursor-pointer disabled:opacity-40"
              :style="{ background: 'var(--color-accent)', color: '#fff' }"
            >
              {{ loading ? '验证中...' : '验证并恢复' }}
            </button>
          </div>
        </div>
      </div>

      <!-- ========== Step: Show (backup) ========== -->
      <div v-else-if="step === 'show'" class="text-center">
        <button
          @click="backToStart"
          class="flex items-center gap-1 text-xs border-none cursor-pointer bg-transparent mb-4"
          :style="{ color: 'var(--color-text-tertiary)' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-secondary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-tertiary)' }"
        >
          ← 返回
        </button>
        <div class="text-4xl mb-4">📝</div>
        <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          请备份你的助记词
        </h2>
        <!-- Fixed HTML rendering with v-html -->
        <p
          class="text-xs mb-5"
          :style="{ color: 'var(--color-text-secondary)' }"
          v-html="'这 12 个单词是恢复密码库的 <strong>唯一方式</strong>。<br />请抄写下来或保存在安全的地方，<strong>不要截屏</strong>。'"
        />
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
              {{ i + 1 }}. {{ word }}
            </span>
          </div>
          <div class="mt-3 text-xs text-center" :style="{ color: 'var(--color-text-tertiary)' }">
            {{ seedInfo }}
          </div>
        </div>
        <button
          @click="handleShowDone"
          class="w-full py-2.5 rounded-lg text-sm font-medium border-none cursor-pointer"
          :style="{ background: 'var(--color-accent)', color: '#fff' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          我已备份，开始验证
        </button>
      </div>

      <!-- ========== Step: Verify ========== -->
      <div v-else-if="step === 'verify'">
        <button
          @click="isImport ? backToStart() : (step = 'show')"
          class="flex items-center gap-1 text-xs border-none cursor-pointer bg-transparent mb-4"
          :style="{ color: 'var(--color-text-tertiary)' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-secondary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-tertiary)' }"
        >
          ← 返回
        </button>

        <h2 class="text-lg font-semibold mb-1 text-center" :style="{ color: 'var(--color-text-primary)' }">
          验证你的助记词
        </h2>
        <p class="text-xs mb-4 text-center" :style="{ color: 'var(--color-text-secondary)' }">
          从下方选项中选出第 <strong>{{ verifyIndex + 1 }}</strong> 个单词
        </p>

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
            验证全部 12 个单词
          </button>
        </div>

        <p v-if="verifyError" class="text-xs text-center" :style="{ color: 'var(--color-danger)' }">
          顺序不对，请仔细回想你的助记词后重新选择！
        </p>
      </div>

      <!-- ========== Step: Done ========== -->
      <div v-else-if="step === 'done'" class="text-center">
        <div class="text-4xl mb-4">✅</div>
        <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--color-text-primary)' }">
          助记词已验证！
        </h2>
        <p class="text-xs mb-6" :style="{ color: 'var(--color-text-secondary)' }">
          请务必将你的 12 个助记词保存在安全的地方。<br />
          这是恢复密码库的 <strong>唯一方式</strong>，丢失后无法找回。
        </p>
        <div
          class="rounded-lg p-4 mb-6 text-left"
          :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
        >
          <div class="text-xs font-semibold mb-2" :style="{ color: 'var(--color-text-secondary)' }">
            你的 12 位助记词
          </div>
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="(word, i) in vault.words"
              :key="i"
              class="px-2 py-0.5 rounded text-xs font-medium"
              :style="{ background: 'var(--color-surface-tertiary)', color: 'var(--color-text-primary)' }"
            >
              {{ i + 1 }}. {{ word }}
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
          进入密码库
        </button>
      </div>
    </div>
  </div>
</template>
