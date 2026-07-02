import { useLocaleStore } from '../stores/locale'
import { messages } from '../i18n'

export function useT() {
  const localeStore = useLocaleStore()
  function t(key: string, params?: Record<string, string | number>): string {
    const msg = messages[localeStore.locale]?.[key]
    if (!msg) return key
    if (!params) return msg
    return msg.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? ''))
  }
  return { t }
}
