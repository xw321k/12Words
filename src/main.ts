import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useVaultStore } from './stores/vault'
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
    await vault.initialize()
    if (!vault.isInitialized) {
      next({ name: 'onboarding' })
      return
    }
    // Check if vault file exists on disk
    const hasFile = await vault.vaultExists()
    if (!hasFile) {
      // Has mnemonic but no vault file — redirect to onboarding for import
      next({ name: 'onboarding' })
      return
    }
  }
  next()
})

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)
app.mount('#app')
