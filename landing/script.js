const button = document.getElementById('themeToggle')

function currentTheme() {
  return document.documentElement.dataset.theme === 'clay-light' ? 'clay-light' : 'clay-dark'
}

function setTheme(theme) {
  document.documentElement.dataset.theme = theme
  try {
    localStorage.setItem('vidboost-theme', theme)
  } catch (e) {}
  button.setAttribute('aria-label', theme === 'clay-light' ? 'Aktifkan dark mode' : 'Aktifkan light mode')
}

setTheme(currentTheme())

button.addEventListener('click', () => {
  setTheme(currentTheme() === 'clay-light' ? 'clay-dark' : 'clay-light')
})
