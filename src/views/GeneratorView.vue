<script setup lang="ts">
import { ref } from 'vue'
import { useVaultStore } from '../stores/vault'

const vault = useVaultStore()
const length = ref(20)
const useNumbers = ref(true)
const useSymbols = ref(true)
const generated = ref('')
const copied = ref(false)

function handleGenerate() {
  vault.generatePassword(length.value, useNumbers.value, useSymbols.value)
    .then(pwd => { generated.value = pwd; copied.value = false })
}

function handleCopy() {
  if (!generated.value) return
  vault.copyToClipboard(generated.value)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
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
        密码生成器
      </h1>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-8">
      <div
        class="max-w-md mx-auto rounded-xl p-6"
        :style="{ background: 'var(--color-surface)', border: '1px solid var(--color-border)' }"
      >
        <!-- Generated password display -->
        <div
          class="mb-6 p-3 rounded-lg text-center font-mono text-lg break-all select-all min-h-[48px] flex items-center justify-center"
          :style="{
            background: 'var(--color-surface-secondary)',
            border: '1px solid var(--color-border)',
            color: generated ? 'var(--color-text-primary)' : 'var(--color-text-tertiary)',
          }"
        >
          {{ generated || '点击生成' }}
        </div>

        <!-- Length slider -->
        <div class="mb-5">
          <div class="flex justify-between items-center mb-2">
            <span class="text-xs" :style="{ color: 'var(--color-text-secondary)' }">长度</span>
            <span class="text-xs font-mono" :style="{ color: 'var(--color-text-primary)' }">{{ length }}</span>
          </div>
          <input
            v-model.number="length"
            type="range"
            min="4"
            max="128"
            class="w-full h-1.5 rounded-full appearance-none cursor-pointer"
            :style="{
              background: 'var(--color-surface-tertiary)',
              accentColor: 'var(--color-accent)',
            }"
          />
          <div class="flex justify-between text-[10px] mt-1" :style="{ color: 'var(--color-text-tertiary)' }">
            <span>4</span>
            <span>128</span>
          </div>
        </div>

        <!-- Options -->
        <div class="space-y-3 mb-6">
          <label class="flex items-center gap-3 cursor-pointer">
            <input
              v-model="useNumbers"
              type="checkbox"
              class="w-3.5 h-3.5 rounded cursor-pointer"
              :style="{ accentColor: 'var(--color-accent)' }"
            />
            <span class="text-xs" :style="{ color: 'var(--color-text-primary)' }">包含数字 (0-9)</span>
          </label>
          <label class="flex items-center gap-3 cursor-pointer">
            <input
              v-model="useSymbols"
              type="checkbox"
              class="w-3.5 h-3.5 rounded cursor-pointer"
              :style="{ accentColor: 'var(--color-accent)' }"
            />
            <span class="text-xs" :style="{ color: 'var(--color-text-primary)' }">包含符号 (!@#$%...)</span>
          </label>
        </div>

        <!-- Buttons -->
        <div class="flex gap-2">
          <button
            @click="handleGenerate"
            class="flex-1 py-2 rounded-lg text-xs font-medium border-none cursor-pointer transition-colors duration-100"
            :style="{ background: 'var(--color-accent)', color: '#fff' }"
            @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent-hover)'"
            @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'var(--color-accent)'"
          >
            生成
          </button>
          <button
            @click="handleCopy"
            :disabled="!generated"
            class="flex-1 py-2 rounded-lg text-xs font-medium border cursor-pointer transition-colors duration-100 disabled:opacity-40 disabled:cursor-not-allowed"
            :style="{
              borderColor: 'var(--color-border)',
              color: copied ? 'var(--color-success)' : 'var(--color-text-secondary)',
              background: 'transparent',
            }"
            @mouseenter="(e) => { if (!copied) (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)' }"
            @mouseleave="(e) => { (e.currentTarget as HTMLElement).style.background = 'transparent' }"
          >
            {{ copied ? '已复制 ✓' : '复制到剪贴板' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
