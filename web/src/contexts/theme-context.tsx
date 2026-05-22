"use client"

import React, { createContext, useContext, useEffect, useState, useCallback } from "react"
import { api } from "@/lib/api"
import {
  generateDaisyuiCss,
  generateShadcnCss,
  type CustomThemeColors,
  type DaisyuiPreset,
} from "@/lib/theme-colors"

export interface FontConfig {
  display?: { url?: string; family?: string }
  heading?: { url?: string; family?: string }
  body?: { url?: string; family?: string }
}

export interface ThemeConfig {
  name: string
  tagline: string
  logoUrl: string | null
  daisyuiPreset: DaisyuiPreset | "custom"
  daisyuiCustomTheme: CustomThemeColors
  customCss: string
  fontConfig: FontConfig
}

interface ThemeContextType {
  config: ThemeConfig | null
  isLoading: boolean
  refresh: () => Promise<void>
}

const DEFAULT_CONFIG: ThemeConfig = {
  name: "OpenHack",
  tagline: "Build something amazing",
  logoUrl: null,
  daisyuiPreset: "light",
  daisyuiCustomTheme: {},
  customCss: "",
  fontConfig: {},
}

const DARK_PRESETS = [
  "dark",
  "night",
  "dracula",
  "black",
  "luxury",
  "business",
  "coffee",
  "dim",
  "winter",
  "sunset",
  "synthwave",
]

const ThemeContext = createContext<ThemeContextType>({
  config: DEFAULT_CONFIG,
  isLoading: true,
  refresh: async () => {},
})

export function useTheme() {
  return useContext(ThemeContext)
}

/**
 * Read DaisyUI CSS variables from the document and convert them to
 * shadcn HSL variables. Works for both built-in presets and custom themes.
 */
function buildColorBridge(): string {
  const testEl = document.createElement("div")
  testEl.style.display = "none"
  document.body.appendChild(testEl)

  const vars: Record<string, string> = {}
  const daisyVars = [
    ["--p", "--primary"],
    ["--pf", "--primary-foreground"],
    ["--s", "--secondary"],
    ["--sf", "--secondary-foreground"],
    ["--a", "--accent"],
    ["--af", "--accent-foreground"],
    ["--n", "--neutral"],
    ["--nf", "--neutral-foreground"],
    ["--b1", "--background"],
    ["--bc", "--foreground"],
    ["--in", "--info"],
    ["--su", "--success"],
    ["--wa", "--warning"],
    ["--er", "--destructive"],
    ["--er-f", "--destructive-foreground"],
  ]

  for (const [daisyVar, shadcnVar] of daisyVars) {
    testEl.style.setProperty("color", `var(${daisyVar})`)
    const computed = getComputedStyle(testEl).color
    if (computed && computed !== "rgba(0, 0, 0, 0)") {
      vars[shadcnVar] = computed
    }
  }

  document.body.removeChild(testEl)

  // Convert rgb() strings to HSL for shadcn
  const hslVars: Record<string, string> = {}
  for (const [key, rgbStr] of Object.entries(vars)) {
    const match = rgbStr.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/)
    if (match) {
      const r = parseInt(match[1]) / 255
      const g = parseInt(match[2]) / 255
      const b = parseInt(match[3]) / 255
      const hsl = rgbToHsl(r, g, b)
      hslVars[key] = `${hsl.h} ${hsl.s}% ${hsl.l}%`
    }
  }

  // Map additional shadcn variables that DaisyUI doesn't have exact matches for
  const cssLines = [
    `:root {`,
    `  --background: ${hslVars["--background"] || "0 0% 100%"};`,
    `  --foreground: ${hslVars["--foreground"] || "222.2 84% 4.9%"};`,
    `  --card: ${hslVars["--background"] || "0 0% 100%"};`,
    `  --card-foreground: ${hslVars["--foreground"] || "222.2 84% 4.9%"};`,
    `  --popover: ${hslVars["--background"] || "0 0% 100%"};`,
    `  --popover-foreground: ${hslVars["--foreground"] || "222.2 84% 4.9%"};`,
    `  --primary: ${hslVars["--primary"] || "221.2 83.2% 53.3%"};`,
    `  --primary-foreground: ${hslVars["--primary-foreground"] || "210 40% 98%"};`,
    `  --secondary: ${hslVars["--secondary"] || "210 40% 96.1%"};`,
    `  --secondary-foreground: ${hslVars["--secondary-foreground"] || "222.2 47.4% 11.2%"};`,
    `  --muted: ${hslVars["--neutral"] || "210 40% 96.1%"};`,
    `  --muted-foreground: ${hslVars["--neutral-foreground"] || "215.4 16.3% 46.9%"};`,
    `  --accent: ${hslVars["--accent"] || "210 40% 96.1%"};`,
    `  --accent-foreground: ${hslVars["--accent-foreground"] || "222.2 47.4% 11.2%"};`,
    `  --destructive: ${hslVars["--destructive"] || "0 84.2% 60.2%"};`,
    `  --destructive-foreground: ${hslVars["--destructive-foreground"] || "210 40% 98%"};`,
    `  --border: ${hslVars["--neutral"] || "214.3 31.8% 91.4%"};`,
    `  --input: ${hslVars["--neutral"] || "214.3 31.8% 91.4%"};`,
    `  --ring: ${hslVars["--primary"] || "221.2 83.2% 53.3%"};`,
    `}`,
  ]

  return cssLines.join("\n")
}

function rgbToHsl(r: number, g: number, b: number) {
  const max = Math.max(r, g, b)
  const min = Math.min(r, g, b)
  const l = (max + min) / 2
  let h = 0
  let s = 0

  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0)
        break
      case g:
        h = (b - r) / d + 2
        break
      case b:
        h = (r - g) / d + 4
        break
    }
    h /= 6
  }

  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) }
}

function injectFontLink(id: string, url: string) {
  if (!url) return
  let link = document.getElementById(id) as HTMLLinkElement | null
  if (!link) {
    link = document.createElement("link")
    link.id = id
    link.rel = "stylesheet"
    document.head.appendChild(link)
  }
  link.href = url
}

function removeFontLink(id: string) {
  const link = document.getElementById(id)
  if (link) link.remove()
}

function injectFontVars(fontConfig: FontConfig) {
  let style = document.getElementById("hackathon-font-vars") as HTMLStyleElement | null
  if (!style) {
    style = document.createElement("style")
    style.id = "hackathon-font-vars"
    document.head.appendChild(style)
  }

  const display = fontConfig.display?.family || ""
  const heading = fontConfig.heading?.family || ""
  const body = fontConfig.body?.family || ""

  style.textContent = `
    :root {
      --font-display: ${display || "ui-sans-serif, system-ui, sans-serif"};
      --font-heading: ${heading || "ui-sans-serif, system-ui, sans-serif"};
      --font-body: ${body || "ui-sans-serif, system-ui, sans-serif"};
    }
  `
}

export function ThemeProvider({ children, initialConfig }: { children: React.ReactNode; initialConfig?: ThemeConfig }) {
  const [config, setConfig] = useState<ThemeConfig | null>(initialConfig || null)
  const [isLoading, setIsLoading] = useState(!initialConfig)

  const refresh = useCallback(async () => {
    try {
      const data = await api.get("/api/core/info")
      const newConfig: ThemeConfig = {
        name: data.name || DEFAULT_CONFIG.name,
        tagline: data.tagline || DEFAULT_CONFIG.tagline,
        logoUrl: data.logo_url || null,
        daisyuiPreset: data.daisyui_theme_preset || "light",
        daisyuiCustomTheme: data.daisyui_custom_theme || {},
        customCss: data.custom_css || "",
        fontConfig: data.font_config || {},
      }
      setConfig(newConfig)
    } catch {
      setConfig(DEFAULT_CONFIG)
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    if (!initialConfig) {
      refresh()
    }
  }, [refresh, initialConfig])

  // Apply theme to document
  useEffect(() => {
    if (!config) return

    const html = document.documentElement

    // Set DaisyUI data-theme attribute
    html.setAttribute("data-theme", config.daisyuiPreset)

    // Sync dark mode for shadcn
    const isDark = DARK_PRESETS.includes(config.daisyuiPreset as string)
    if (isDark) {
      html.classList.add("dark")
    } else {
      html.classList.remove("dark")
    }

    // Remove old injected styles
    const oldDaisy = document.getElementById("daisyui-custom-theme")
    const oldShadcn = document.getElementById("shadcn-custom-theme")
    const oldCustom = document.getElementById("hackathon-custom-css")
    const oldColorBridge = document.getElementById("color-bridge")
    if (oldDaisy) oldDaisy.remove()
    if (oldShadcn) oldShadcn.remove()
    if (oldCustom) oldCustom.remove()
    if (oldColorBridge) oldColorBridge.remove()

    // Inject custom DaisyUI theme variables if preset is "custom"
    if (config.daisyuiPreset === "custom" && Object.keys(config.daisyuiCustomTheme).length > 0) {
      const daisyStyle = document.createElement("style")
      daisyStyle.id = "daisyui-custom-theme"
      daisyStyle.textContent = generateDaisyuiCss(config.daisyuiCustomTheme)
      document.head.appendChild(daisyStyle)

      const shadcnStyle = document.createElement("style")
      shadcnStyle.id = "shadcn-custom-theme"
      shadcnStyle.textContent = generateShadcnCss(config.daisyuiCustomTheme)
      document.head.appendChild(shadcnStyle)
    } else if (config.daisyuiPreset !== "custom") {
      // For built-in presets, build a color bridge from DaisyUI -> shadcn
      // Use a small timeout to let DaisyUI apply its variables first
      const timer = setTimeout(() => {
        const bridgeStyle = document.createElement("style")
        bridgeStyle.id = "color-bridge"
        bridgeStyle.textContent = buildColorBridge()
        document.head.appendChild(bridgeStyle)
      }, 50)
      return () => clearTimeout(timer)
    }

    // Inject custom CSS
    if (config.customCss) {
      const customStyle = document.createElement("style")
      customStyle.id = "hackathon-custom-css"
      customStyle.textContent = config.customCss
      document.head.appendChild(customStyle)
    }

    // Inject font links
    injectFontLink("font-display-link", config.fontConfig.display?.url || "")
    injectFontLink("font-heading-link", config.fontConfig.heading?.url || "")
    injectFontLink("font-body-link", config.fontConfig.body?.url || "")
    injectFontVars(config.fontConfig)
  }, [config])

  return (
    <ThemeContext.Provider value={{ config: config || DEFAULT_CONFIG, isLoading, refresh }}>
      {children}
    </ThemeContext.Provider>
  )
}
