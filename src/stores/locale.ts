import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Locale } from '../i18n'

export const useLocaleStore = defineStore('locale', () => {
  const locale = ref<Locale>('zh')

  function setLocale(l: Locale) {
    locale.value = l
    localStorage.setItem('locale', l)
  }

  function init() {
    const saved = localStorage.getItem('locale') as Locale | null
    if (saved === 'en' || saved === 'zh') {
      locale.value = saved
    }
  }

  return { locale, setLocale, init }
})
