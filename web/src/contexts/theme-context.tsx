"use client"

import React, { createContext, useContext, useEffect, useState, useCallback, useRef } from "react"
import { api } from "@/lib/api"
import {
  generateDaisyuiCss,
  generateShadcnCss,
  oklchStrToHsl,
  DARK_PRESETS,
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

const ThemeContext = createContext<ThemeContextType>({
  config: DEFAULT_CONFIG,
  isLoading: true,
  refresh: async () => {},
})

export function useTheme() {
  return useContext(ThemeContext)
}

/**
 * Read DaisyUI v4 CSS variables from the document and convert them to
 * shadcn HSL variables. DaisyUI v4 uses OKLCH colors under
 * `--color-*` names; we read the raw values and convert via culori.
 */
export function buildColorBridge(): string {
  const root = getComputedStyle(document.documentElement)

  const mappings: [string, string][] = [
    // Backgrounds — use base scale so secondary/muted are subtle grays
    ["--color-base-100", "--background"],
    ["--color-base-100", "--card"],
    ["--color-base-100", "--popover"],
    ["--color-base-200", "--secondary"],
    ["--color-base-200", "--muted"],
    ["--color-base-200", "--input"],
    ["--color-base-300", "--border"],

    // Foregrounds — base-content for main text, neutral (dark) for muted text
    ["--color-base-content", "--foreground"],
    ["--color-base-content", "--card-foreground"],
    ["--color-base-content", "--popover-foreground"],
    ["--color-base-content", "--secondary-foreground"],
    ["--color-neutral", "--muted-foreground"],

    // Accents — use DaisyUI semantic colors directly
    ["--color-primary", "--primary"],
    ["--color-primary-content", "--primary-foreground"],
    ["--color-accent", "--accent"],
    ["--color-accent-content", "--accent-foreground"],
    ["--color-info", "--info"],
    ["--color-success", "--success"],
    ["--color-warning", "--warning"],
    ["--color-error", "--destructive"],
    ["--color-error-content", "--destructive-foreground"],

    // Ring matches primary for consistency
    ["--color-primary", "--ring"],
  ]

  const hslVars: Record<string, string> = {}

  for (const [daisyVar, shadcnVar] of mappings) {
    const raw = root.getPropertyValue(daisyVar).trim()
    if (!raw) continue
    const hslVal = oklchStrToHsl(raw)
    if (hslVal) {
      hslVars[shadcnVar] = hslVal
    }
  }

  const cssLines = [
    `:root {`,
    `  --background: ${hslVars["--background"] || "0 0% 100%"};`,
    `  --foreground: ${hslVars["--foreground"] || "222.2 84% 4.9%"};`,
    `  --card: ${hslVars["--card"] || "0 0% 100%"};`,
    `  --card-foreground: ${hslVars["--card-foreground"] || "222.2 84% 4.9%"};`,
    `  --popover: ${hslVars["--popover"] || "0 0% 100%"};`,
    `  --popover-foreground: ${hslVars["--popover-foreground"] || "222.2 84% 4.9%"};`,
    `  --primary: ${hslVars["--primary"] || "221.2 83.2% 53.3%"};`,
    `  --primary-foreground: ${hslVars["--primary-foreground"] || "210 40% 98%"};`,
    `  --secondary: ${hslVars["--secondary"] || "210 40% 96.1%"};`,
    `  --secondary-foreground: ${hslVars["--secondary-foreground"] || "222.2 47.4% 11.2%"};`,
    `  --muted: ${hslVars["--muted"] || "210 40% 96.1%"};`,
    `  --muted-foreground: ${hslVars["--muted-foreground"] || "215.4 16.3% 46.9%"};`,
    `  --accent: ${hslVars["--accent"] || "210 40% 96.1%"};`,
    `  --accent-foreground: ${hslVars["--accent-foreground"] || "222.2 47.4% 11.2%"};`,
    `  --destructive: ${hslVars["--destructive"] || "0 84.2% 60.2%"};`,
    `  --destructive-foreground: ${hslVars["--destructive-foreground"] || "210 40% 98%"};`,
    `  --border: ${hslVars["--border"] || "214.3 31.8% 91.4%"};`,
    `  --input: ${hslVars["--input"] || "214.3 31.8% 91.4%"};`,
    `  --ring: ${hslVars["--ring"] || "221.2 83.2% 53.3%"};`,
    `}`,
  ]

  return cssLines.join("\n")
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
  const colorBridgeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

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

    // Clear any pending color bridge timer from previous config
    if (colorBridgeTimerRef.current) {
      clearTimeout(colorBridgeTimerRef.current)
      colorBridgeTimerRef.current = null
    }

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
      colorBridgeTimerRef.current = setTimeout(() => {
        colorBridgeTimerRef.current = null
        const bridgeStyle = document.createElement("style")
        bridgeStyle.id = "color-bridge"
        bridgeStyle.textContent = buildColorBridge()
        document.head.appendChild(bridgeStyle)
      }, 50)
    }

    // Inject custom CSS (always reachable — applies to all presets)
    if (config.customCss) {
      const customStyle = document.createElement("style")
      customStyle.id = "hackathon-custom-css"
      customStyle.textContent = config.customCss
      document.head.appendChild(customStyle)
    }

    // Inject font links (always reachable — applies to all presets)
    injectFontLink("font-display-link", config.fontConfig.display?.url || "")
    injectFontLink("font-heading-link", config.fontConfig.heading?.url || "")
    injectFontLink("font-body-link", config.fontConfig.body?.url || "")
    injectFontVars(config.fontConfig)

    return () => {
      if (colorBridgeTimerRef.current) {
        clearTimeout(colorBridgeTimerRef.current)
        colorBridgeTimerRef.current = null
      }
    }
  }, [config])

  return (
    <ThemeContext.Provider value={{ config: config || DEFAULT_CONFIG, isLoading, refresh }}>
      {children}
    </ThemeContext.Provider>
  )
}
