<script setup lang="ts">
import { ref } from 'vue'
import { useVaultStore } from '../stores/vault'

const vault = useVaultStore()

// Steps: 'generate' → 'verify' → 'done'
type Step = 'generate' | 'verify' | 'done'
const step = ref<Step>('generate')
const loading = ref(false)
const error = ref('')
const seedInfo = ref('')

// Verification: user must click words in order
const verifyWords = ref<string[]>([])
const verifyIndex = ref(0)
const shuffledWords = ref<string[]>([])
const wrongAttempt = ref(false)

async function handleGenerate() {
  loading.value = true
  error.value = ''
  try {
    await vault.generate()
    seedInfo.value = `Seed (SHA256): ${vault.userId.slice(0, 16)}...`
    // Prepare verification: shuffle, make sure it's different from original order
    const words = [...vault.words]
    // Fisher-Yates shuffle
    for (let i = words.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [words[i], words[j]] = [words[j], words[i]]
    }
    // Check it's not accidentally the same order
    const isSame = words.every((w, i) => w === vault.words[i])
    if (isSame) {
      // Swap first two
      ;[words[0], words[1]] = [words[1], words[0]]
    }
    shuffledWords.value = words
    verifyWords.value = []
    verifyIndex.value = 0
    step.value = 'verify'
  } catch (e: any) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function handleWordClick(word: string) {
  if (verifyIndex.value >= 12) return
  const expected = vault.words[verifyIndex.value]
  if (word === expected) {
    wrongAttempt.value = false
    verifyWords.value.push(word)
    verifyIndex.value++
    // Remove from available
    const idx = shuffledWords.value.indexOf(word)
    if (idx !== -1) shuffledWords.value.splice(idx, 1)
    // Check if all verified
    if (verifyIndex.value === 12) {
      step.value = 'done'
    }
  } else {
    wrongAttempt.value = true
    setTimeout(() => { wrongAttempt.value = false }, 500)
  }
}

function handleStartOver() {
  // Re-shuffle and reset
  const words = [...vault.words]
  for (let i = words.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [words[i], words[j]] = [words[j], words[i]]
  }
  shuffledWords.value = words
  verifyWords.value = []
  verifyIndex.value = 0
  step.value = 'verify'
}

function handleDone() {
  // Router will redirect to main view
}
</script>

<template>
  <div
    class="h-screen w-screen flex items-center justify-center"
    :style="{ background: 'var(--color-surface-secondary)' }"
  >
    <div class="w-full max-w-lg mx-6">
      <!-- Step: Generate -->
      <div v-if="step === 'generate'" class="text-center">
        <div class="text-4xl mb-4">🔐</div>
        <h1
          class="text-xl font-semibold mb-2"
          :style="{ color: 'var(--color-text-primary)' }"
        >
          欢迎使用 12Words
        </h1>
        <p
          class="text-sm mb-8"
          :style="{ color: 'var(--color-text-secondary)' }"
        >
          点击下方按钮生成你的唯一 12 位助记词
          <br />这是访问你密码库的 <strong>唯一钥匙</strong>
        </p>

        <div v-if="error" class="mb-4 text-xs" :style="{ color: 'var(--color-danger)' }">
          {{ error }}
        </div>

        <button
          @click="handleGenerate"
          :disabled="loading"
          class="w-full py-2.5 rounded-lg text-sm font-medium transition-colors duration-100 border-none cursor-pointer"
          :style="{
            background: 'var(--color-accent)',
            color: '#fff',
            opacity: loading ? 0.6 : 1,
          }"
          @mouseenter="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { if (!loading) (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          {{ loading ? '生成中...' : '生成助记词' }}
        </button>
      </div>

      <!-- Step: Verify -->
      <div v-else-if="step === 'verify'">
        <h2
          class="text-lg font-semibold mb-1 text-center"
          :style="{ color: 'var(--color-text-primary)' }"
        >
          验证你的助记词
        </h2>
        <p
          class="text-xs mb-5 text-center"
          :style="{ color: 'var(--color-text-secondary)' }"
        >
          请按<strong>正确顺序</strong>点击下方单词
        </p>

        <!-- Progress bar -->
        <div
          class="h-1 rounded-full mb-5"
          :style="{ background: 'var(--color-surface-tertiary)' }"
        >
          <div
            class="h-full rounded-full transition-all duration-200"
            :style="{
              width: `${(verifyIndex / 12) * 100}%`,
              background: 'var(--color-accent)',
            }"
          />
        </div>

        <!-- Verified words display -->
        <div
          class="flex flex-wrap gap-1.5 mb-4 min-h-[40px] p-3 rounded-lg"
          :style="{
            background: 'var(--color-surface)',
            border: wrongAttempt ? '1px solid var(--color-danger)' : '1px solid var(--color-border)',
          }"
        >
          <span
            v-for="(w, i) in verifyWords"
            :key="i"
            class="px-2 py-0.5 rounded text-xs font-medium"
            :style="{
              background: 'var(--color-accent)',
              color: '#fff',
            }"
          >
            {{ w }}
          </span>
          <span
            v-if="verifyIndex < 12"
            class="px-2 py-0.5 rounded text-xs"
            :style="{ color: 'var(--color-text-tertiary)' }"
          >
            第 {{ verifyIndex + 1 }} 个
          </span>
        </div>

        <!-- Word buttons -->
        <div class="flex flex-wrap gap-2 mb-5">
          <button
            v-for="word in shuffledWords"
            :key="word"
            @click="handleWordClick(word)"
            class="px-3 py-1.5 rounded-md text-xs font-medium border cursor-pointer transition-all duration-100"
            :style="{
              background: 'var(--color-surface)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text-primary)',
            }"
            @mouseenter="(e) => {
              (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-accent)'
            }"
            @mouseleave="(e) => {
              (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)'
            }"
          >
            {{ word }}
          </button>
        </div>

        <p
          v-if="wrongAttempt"
          class="text-xs mb-3"
          :style="{ color: 'var(--color-danger)' }"
        >
          顺序不对，仔细想想！
        </p>

        <button
          @click="handleStartOver"
          class="text-xs border-none cursor-pointer bg-transparent"
          :style="{ color: 'var(--color-text-tertiary)' }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-secondary)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.color = 'var(--color-text-tertiary)' }"
        >
          重新打乱重来
        </button>
      </div>

      <!-- Step: Done -->
      <div v-else-if="step === 'done'" class="text-center">
        <div class="text-4xl mb-4">✅</div>
        <h2
          class="text-lg font-semibold mb-2"
          :style="{ color: 'var(--color-text-primary)' }"
        >
          助记词已验证！
        </h2>
        <p
          class="text-xs mb-6"
          :style="{ color: 'var(--color-text-secondary)' }"
        >
          请务必将你的 12 个助记词保存在安全的地方。<br />
          这是恢复密码库的 <strong>唯一方式</strong>，丢失后无法找回。
        </p>

        <!-- Mnemonic card for backup -->
        <div
          class="rounded-lg p-4 mb-6 text-left"
          :style="{
            background: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
          }"
        >
          <div class="text-xs font-semibold mb-2" :style="{ color: 'var(--color-text-secondary)' }">
            你的 12 位助记词
          </div>
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="(word, i) in vault.words"
              :key="i"
              class="px-2 py-0.5 rounded text-xs font-medium"
              :style="{
                background: 'var(--color-surface-tertiary)',
                color: 'var(--color-text-primary)',
              }"
            >
              {{ i + 1 }}. {{ word }}
            </span>
          </div>
          <div class="mt-3 text-xs" :style="{ color: 'var(--color-text-tertiary)' }">
            {{ seedInfo }}
          </div>
        </div>

        <button
          @click="handleDone"
          class="w-full py-2.5 rounded-lg text-sm font-medium transition-colors duration-100 border-none cursor-pointer"
          :style="{
            background: 'var(--color-accent)',
            color: '#fff',
          }"
          @mouseenter="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)' }"
          @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)' }"
        >
          进入密码库
        </button>
      </div>
    </div>
  </div>
</template>
