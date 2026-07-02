<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useVaultStore } from '../stores/vault'

const router = useRouter()
const route = useRoute()
const vault = useVaultStore()

type NavItem = {
  id: string
  label: string
  icon: string
  route: string
  category: string
}

const navItems: NavItem[] = [
  { id: 'all', label: '全部', icon: '📋', route: '/passwords', category: '' },
  { id: 'login', label: '账号密码', icon: '🔑', route: '/passwords', category: 'Login' },
  { id: 'apikey', label: 'API密钥', icon: '🔌', route: '/passwords', category: 'ApiKey' },
  { id: 'license', label: '软件卡密', icon: '🏷️', route: '/passwords', category: 'License' },
  { id: 'card', label: '银行卡', icon: '💳', route: '/passwords', category: 'Card' },
  { id: 'identity', label: '证件', icon: '🪪', route: '/passwords', category: 'Identity' },
  { id: 'crypto', label: '钱包私钥', icon: '₿', route: '/passwords', category: 'Crypto' },
  { id: 'note', label: '加密便签', icon: '📄', route: '/passwords', category: 'Note' },
]

// Add separator, generator, settings
const bottomItems: NavItem[] = [
  { id: 'generator', label: '生成器', icon: '⚡', route: '/generator', category: '' },
  { id: 'settings', label: '设置', icon: '⚙', route: '/settings', category: '' },
]

function setCategory(cat: string) {
  vault.filterCategory = cat
  router.push('/passwords')
}

const activeCategory = computed(() => vault.filterCategory)
</script>

<template>
  <nav
    class="w-56 flex-shrink-0 flex flex-col border-r"
    :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface-secondary)' }"
  >
    <div
      class="h-14 flex items-center px-5 border-b text-sm font-semibold tracking-wide flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)' }"
    >
      <span class="mr-2">🔐</span>
      12Words
    </div>

    <!-- Category filters -->
    <div class="flex-1 overflow-y-auto px-2 py-3 space-y-0.5">
      <div
        v-for="item in navItems"
        :key="item.id"
        @click="setCategory(item.category)"
        class="flex items-center gap-3 px-3 py-1.5 rounded-md text-xs cursor-pointer transition-colors duration-100"
        :style="{
          background: activeCategory === item.category && route.name === 'passwords' ? 'var(--color-surface)' : 'transparent',
          color: activeCategory === item.category && route.name === 'passwords' ? 'var(--color-accent)' : 'var(--color-text-secondary)',
        }"
        @mouseenter="(e) => {
          if (!(activeCategory === item.category && route.name === 'passwords'))
            (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)'
        }"
        @mouseleave="(e) => {
          if (!(activeCategory === item.category && route.name === 'passwords'))
            (e.currentTarget as HTMLElement).style.background = 'transparent'
        }"
      >
        <span class="text-sm">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </div>

      <div class="my-2 border-t" :style="{ borderColor: 'var(--color-border)' }" />

      <div
        v-for="item in bottomItems"
        :key="item.id"
        @click="router.push(item.route)"
        class="flex items-center gap-3 px-3 py-1.5 rounded-md text-xs cursor-pointer transition-colors duration-100"
        :style="{
          background: route.name === item.id ? 'var(--color-surface)' : 'transparent',
          color: route.name === item.id ? 'var(--color-accent)' : 'var(--color-text-secondary)',
        }"
        @mouseenter="(e) => {
          if (route.name !== item.id)
            (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)'
        }"
        @mouseleave="(e) => {
          if (route.name !== item.id)
            (e.currentTarget as HTMLElement).style.background = 'transparent'
        }"
      >
        <span class="text-sm">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </div>
    </div>

    <div
      class="px-4 py-3 border-t text-xs flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-tertiary)' }"
    >
      v0.1.0
    </div>
  </nav>
</template>
