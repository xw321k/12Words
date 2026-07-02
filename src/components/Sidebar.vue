<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

type NavItem = {
  id: string
  label: string
  icon: string
  route: string
}

const navItems: NavItem[] = [
  { id: 'passwords', label: '密码', icon: '🔑', route: '/passwords' },
  { id: 'generator', label: '生成器', icon: '⚡', route: '/generator' },
  { id: 'settings', label: '设置', icon: '⚙', route: '/settings' },
]

const activeId = computed(() => route.name as string)
</script>

<template>
  <nav
    class="w-56 flex-shrink-0 flex flex-col border-r"
    :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface-secondary)' }"
  >
    <!-- Logo / Brand -->
    <div
      class="h-14 flex items-center px-5 border-b text-sm font-semibold tracking-wide"
      :style="{ borderColor: 'var(--color-border)' }"
    >
      <span class="mr-2">🔐</span>
      12Words
    </div>

    <!-- Navigation -->
    <div class="flex-1 px-2 py-4 space-y-1">
      <button
        v-for="item in navItems"
        :key="item.id"
        @click="router.push(item.route)"
        class="w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors duration-100"
        :style="{
          background: activeId === item.id ? 'var(--color-surface)' : 'transparent',
          color: activeId === item.id ? 'var(--color-accent)' : 'var(--color-text-secondary)',
        }"
        @mouseenter="(e) => {
          if (activeId !== item.id) (e.currentTarget as HTMLElement).style.background = 'var(--color-surface-tertiary)'
        }"
        @mouseleave="(e) => {
          if (activeId !== item.id) (e.currentTarget as HTMLElement).style.background = 'transparent'
        }"
      >
        <span class="text-base">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </button>
    </div>

    <!-- Footer -->
    <div
      class="px-4 py-3 border-t text-xs"
      :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-tertiary)' }"
    >
      v0.1.0
    </div>
  </nav>
</template>
