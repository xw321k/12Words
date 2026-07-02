import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useVaultStore } from './stores/vault'
import { useLocaleStore } from './stores/locale'
import App from './App.vue'
import './style.css'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/passwords',
    },
    {
      path: '/onboarding',
      name: 'onboarding',
      component: () => import('./views/OnboardingView.vue'),
    },
    {
      path: '/passwords',
      name: 'passwords',
      component: () => import('./views/PasswordsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/generator',
      name: 'generator',
      component: () => import('./views/GeneratorView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('./views/SettingsView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

// Navigation guard: redirect to onboarding if not initialized
router.beforeEach(async (to, _from, next) => {
  if (to.meta.requiresAuth) {
    const vault = useVaultStore()
    if (!vault.isInitialized) {
      await vault.initialize()
      if (!vault.isInitialized) {
        next({ name: 'onboarding' })
        return
      }
      // 只有从 localStorage 加载的情况需要检查 vault 文件是否存在
      // 新用户刚完成 onboarding 时 isInitialized 已为 true，不会走到这里
      if (vault.isFromStorage) {
        const hasFile = await vault.vaultExists()
        if (!hasFile) {
          next({ name: 'onboarding' })
          return
        }
      }
    }
    // isInitialized 已为 true（刚完成onboarding），直接放行
  }
  next()
})

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)

// Init locale
const locale = useLocaleStore()
locale.init()

app.mount('#app')
