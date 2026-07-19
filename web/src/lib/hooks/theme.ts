import * as React from "react"

export type ThemePreference = "light" | "dark" | "system"

export const THEME_HEAD_INLINE_SCRIPT = `(() => {
  try {
    const raw = localStorage.getItem("theme");
    const trimmed = typeof raw === "string" ? raw.trim() : "";
    const pref = trimmed === "light" || trimmed === "dark" || trimmed === "system" ? trimmed : "system";
    const sysDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const sysLight = window.matchMedia("(prefers-color-scheme: light)").matches;
    let effective;
    if (pref === "system") {
      if (sysLight && !sysDark) effective = "light";
      else if (sysDark && !sysLight) effective = "dark";
      else if (sysDark && sysLight) effective = "light";
      else effective = "light";
    } else {
      effective = pref;
    }
    const root = document.documentElement;
    root.setAttribute("data-theme-preference", pref);
    root.classList.remove("light", "dark");
    root.classList.add(effective);
    root.setAttribute("data-theme", effective);
    root.style.colorScheme = effective;
  } catch {
  }
})();`

const STORAGE_KEY = "theme"

function readStoredPreference(): ThemePreference {
  if (typeof window === "undefined") {
    return "system"
  }

  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    const stored = typeof raw === "string" ? raw.trim() : ""
    if (stored === "light" || stored === "dark" || stored === "system") {
      return stored
    }
  } catch {
    return "system"
  }

  return "system"
}

export const getThemePreference = (): ThemePreference => readStoredPreference()

export const setThemePreference = (theme: ThemePreference): void => {
  if (typeof window === "undefined") {
    return
  }

  try {
    localStorage.setItem(STORAGE_KEY, theme)
  } catch {
    return
  }
}

let systemDarkMql: MediaQueryList | null = null
let systemLightMql: MediaQueryList | null = null

function getSystemDarkMql(): MediaQueryList {
  if (typeof window === "undefined") {
    throw new Error("getSystemDarkMql requires window")
  }
  if (!systemDarkMql) {
    systemDarkMql = window.matchMedia("(prefers-color-scheme: dark)")
  }
  return systemDarkMql
}

function getSystemLightMql(): MediaQueryList {
  if (typeof window === "undefined") {
    throw new Error("getSystemLightMql requires window")
  }
  if (!systemLightMql) {
    systemLightMql = window.matchMedia("(prefers-color-scheme: light)")
  }
  return systemLightMql
}

export function resolveSystemEffectiveTheme(): "light" | "dark" {
  if (typeof window === "undefined") {
    return "light"
  }
  const d = getSystemDarkMql().matches
  const l = getSystemLightMql().matches
  if (l && !d) {
    return "light"
  }
  if (d && !l) {
    return "dark"
  }
  if (d && l) {
    return "light"
  }
  return "light"
}

export const getEffectiveTheme = (theme: ThemePreference): "light" | "dark" => {
  if (theme === "system") {
    return resolveSystemEffectiveTheme()
  }
  return theme
}

export const getNextTheme = (current: ThemePreference): ThemePreference => {
  if (current === "light") {
    return "dark"
  }
  if (current === "dark") {
    return "system"
  }
  return "light"
}

function applyEffectiveToDocument(effective: "light" | "dark"): void {
  if (typeof document === "undefined") {
    return
  }
  const root = document.documentElement
  root.classList.remove("light", "dark")
  root.classList.add(effective)
  root.setAttribute("data-theme", effective)
  root.style.colorScheme = effective
}

function setThemePreferenceAttribute(preference: ThemePreference): void {
  if (typeof document === "undefined") {
    return
  }
  document.documentElement.setAttribute("data-theme-preference", preference)
}

export function syncRootFromPreference(preference: ThemePreference): void {
  if (typeof window === "undefined") {
    return
  }
  setThemePreferenceAttribute(preference)
  applyEffectiveToDocument(getEffectiveTheme(preference))
}

export const applyThemePreference = (preference: ThemePreference): void => {
  if (typeof window === "undefined") {
    return
  }
  setThemePreference(preference)
  syncRootFromPreference(preference)
  window.dispatchEvent(new CustomEvent("sideout:theme-preference"))
}

let systemListenersAttached = false

function onSystemMediaChange(): void {
  if (readStoredPreference() !== "system") {
    return
  }
  syncRootFromPreference("system")
}

export const initThemeDocument = (): void => {
  if (typeof window === "undefined") {
    return
  }

  const pref = readStoredPreference()
  syncRootFromPreference(pref)

  if (!systemListenersAttached) {
    systemListenersAttached = true
    getSystemDarkMql().addEventListener("change", onSystemMediaChange)
    getSystemLightMql().addEventListener("change", onSystemMediaChange)
  }
}

export function useTheme() {
  const [preference, setPreference] = React.useState<ThemePreference>(() =>
    typeof window === "undefined" ? "system" : getThemePreference()
  )

  const effective = React.useMemo(
    () => getEffectiveTheme(preference),
    [preference]
  )

  React.useEffect(() => {
    initThemeDocument()
  }, [])

  React.useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key === "theme") {
        setPreference(getThemePreference())
      }
    }
    const onCustom = () => {
      setPreference(getThemePreference())
    }
    window.addEventListener("storage", onStorage)
    window.addEventListener("sideout:theme-preference", onCustom)
    return () => {
      window.removeEventListener("storage", onStorage)
      window.removeEventListener("sideout:theme-preference", onCustom)
    }
  }, [])

  const setTheme = React.useCallback((next: ThemePreference) => {
    applyThemePreference(next)
    setPreference(next)
  }, [])

  const cycle = React.useCallback(() => {
    setTheme(getNextTheme(preference))
  }, [preference, setTheme])

  return { preference, effective, setTheme, cycle }
}
