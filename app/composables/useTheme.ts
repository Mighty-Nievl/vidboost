import { ref } from 'vue'
import { THEME_STORAGE_KEY } from '~/utils/vidboost'

export type ThemeName = 'clay-light' | 'clay-dark'

const theme = ref<ThemeName>('clay-light')
let applied = false

function apply(value: ThemeName) {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.theme = value
}

export function useTheme() {
  function setTheme(value: ThemeName) {
    theme.value = value
    apply(value)
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(THEME_STORAGE_KEY, value)
    }
  }

  function toggleTheme() {
    setTheme(theme.value === 'clay-light' ? 'clay-dark' : 'clay-light')
  }

  function initTheme() {
    if (applied) return
    applied = true
    const stored = typeof localStorage !== 'undefined'
      ? localStorage.getItem(THEME_STORAGE_KEY)
      : null
    setTheme(stored === 'clay-dark' || stored === 'clay-light' ? stored : 'clay-light')
  }

  return { theme, setTheme, toggleTheme, initTheme }
}
