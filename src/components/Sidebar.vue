<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useVaultStore } from '../stores/vault'
import { useLocaleStore } from '../stores/locale'
import { categoryLabels, categoryColors } from '../i18n'
import { messages } from '../i18n'

const router = useRouter()
const route = useRoute()
const vault = useVaultStore()
const loc = useLocaleStore()

type NavItem = {
  id: string
  labelKey: string
  icon: string
  route: string
  category: string
}

const defaultItems: NavItem[] = [
  { id: 'all', labelKey: 'sidebar.all', icon: '📋', route: '/passwords', category: '' },
  { id: 'login', labelKey: '', icon: '🔑', route: '/passwords', category: 'Login' },
  { id: 'apikey', labelKey: '', icon: '🔌', route: '/passwords', category: 'ApiKey' },
  { id: 'license', labelKey: '', icon: '🏷️', route: '/passwords', category: 'License' },
  { id: 'card', labelKey: '', icon: '💳', route: '/passwords', category: 'Card' },
  { id: 'identity', labelKey: '', icon: '🆔', route: '/passwords', category: 'Identity' },
  { id: 'crypto', labelKey: '', icon: '💰', route: '/passwords', category: 'Crypto' },
  { id: 'email', labelKey: '', icon: '✉️', route: '/passwords', category: 'Email' },
  { id: 'note', labelKey: '', icon: '📄', route: '/passwords', category: 'Note' },
]

const bottomItems: NavItem[] = [
  { id: 'generator', labelKey: 'sidebar.generator', icon: '⚡', route: '/generator', category: '' },
  { id: 'settings', labelKey: 'sidebar.settings', icon: '⚙', route: '/settings', category: '' },
]

function loadOrder(): NavItem[] {
  try {
    const saved = localStorage.getItem('sidebar_order')
    if (saved) {
      const order: string[] = JSON.parse(saved)
      const map = new Map(defaultItems.map(i => [i.id, i]))
      return order.map(id => map.get(id)).filter(Boolean) as NavItem[]
    }
  } catch {}
  return [...defaultItems]
}

const navItems = ref(loadOrder())
const dragging = ref(false)
const dragSrcIdx = ref(-1)
const overIdx = ref(-1)
const listEl = ref<HTMLElement | null>(null)

function saveOrder() {
  localStorage.setItem('sidebar_order', JSON.stringify(navItems.value.map(i => i.id)))
}

function onMousedown(e: MouseEvent, idx: number) {
  if (idx === 0) return
  dragSrcIdx.value = idx
  dragging.value = true
  overIdx.value = idx
  e.preventDefault()
}

function onMousemove(e: MouseEvent) {
  if (!dragging.value || !listEl.value) return
  const items = listEl.value.querySelectorAll('.drag-item')
  for (let i = 1; i < items.length; i++) {
    const r = items[i].getBoundingClientRect()
    if (e.clientY < r.top + r.height / 2) {
      if (overIdx.value !== i) {
        overIdx.value = i
        const items2 = [...navItems.value]
        const [moved] = items2.splice(dragSrcIdx.value, 1)
        const insertAt = i > dragSrcIdx.value ? i - 1 : i
        items2.splice(insertAt, 0, moved)
        navItems.value = items2
        dragSrcIdx.value = insertAt
      }
      return
    }
  }
}

function onMouseup() {
  if (dragging.value) {
    dragging.value = false
    dragSrcIdx.value = -1
    overIdx.value = -1
    saveOrder()
  }
}

function onClickItem(cat: string) {
  if (dragging.value) return
  setCategory(cat)
}

function t(key: string): string {
  return messages[loc.locale]?.[key] || key
}

function navLabel(item: NavItem): string {
  if (item.labelKey) return t(item.labelKey)
  return categoryLabels[loc.locale]?.[item.category] || item.category
}

function setCategory(cat: string) {
  vault.filterCategory = cat
  router.push('/passwords')
}

function countFor(cat: string): number {
  if (!cat) return vault.entries.length
  return vault.entries.filter(e => e.category === cat).length
}

const activeCategory = computed(() => vault.filterCategory)
</script>

<template>
  <nav
    class="w-56 flex-shrink-0 flex flex-col border-r select-none"
    :style="{ borderColor: 'var(--color-border)', background: 'var(--color-surface-secondary)' }"
    @mouseup="onMouseup"
    @mouseleave="onMouseup"
  >
    <div
      class="h-14 flex items-center px-5 border-b text-sm font-semibold tracking-wide flex-shrink-0"
      :style="{ borderColor: 'var(--color-border)' }"
    >
      <span class="mr-2">🔐</span>
      12Words
    </div>

    <div
      ref="listEl"
      class="flex-1 overflow-y-auto px-2 py-3 space-y-0.5"
      @mousemove="onMousemove"
    >
      <div
        v-for="(item, idx) in navItems"
        :key="item.id"
        @mousedown="onMousedown($event, idx)"
        @click="onClickItem(item.category)"
        class="drag-item flex items-center gap-3 px-3 py-1.5 rounded-md text-xs transition-colors duration-100 cursor-pointer"
        :style="{
          background: activeCategory === item.category && route.name === 'passwords' ? 'var(--color-surface)' : 'transparent',
          color: activeCategory === item.category && route.name === 'passwords' ? 'var(--color-accent)' : 'var(--color-text-secondary)',
          opacity: dragging && dragSrcIdx !== idx ? '0.35' : '1',
          borderLeft: dragging && dragSrcIdx === idx ? '3px solid var(--color-accent)' : '3px solid transparent',
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
        <span v-if="item.id !== 'all'" class="text-[10px] flex-shrink-0 mr-1.5" :style="{ color: 'var(--color-text-tertiary)' }">⠿</span>
        <span class="text-sm flex-shrink-0">{{ item.icon }}</span>
        <span class="truncate flex-1">{{ navLabel(item) }}</span>
        <span v-if="item.id !== 'all'" class="text-[10px] px-1.5 py-0.5 rounded font-medium" :style="{ background: categoryColors[item.category] + '20', color: categoryColors[item.category] }">{{ countFor(item.category) }}</span>
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
        <span>{{ navLabel(item) }}</span>
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
