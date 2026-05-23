/**
 * Color conversion utilities for DaisyUI ↔ shadcn/ui theme bridging.
 *
 * Expected Behavior:
 *   Converts hex colors to OKLCH (for DaisyUI v4 CSS variables) and to
 *   HSL space-separated strings (for shadcn/ui CSS variables). Uses the
 *   culori library for accurate color-space conversions.
 *
 *   hexToOklch("#3b82f6") → "62.3% 0.192 257"
 *   hexToHsl("#3b82f6")  → "217 91% 60%"
 *
 *   generateDaisyuiCss({ primary: "#3b82f6", ... }) → CSS string
 *   generateShadcnCss({ primary: "#3b82f6", ... })  → CSS string
 *
 * Side Effects:
 *   None. Pure functions.
 */

import { parse, oklch, hsl, formatCss } from "culori"

export interface CustomThemeColors {
  primary?: string
  secondary?: string
  accent?: string
  neutral?: string
  "base-100"?: string
  info?: string
  success?: string
  warning?: string
  error?: string
}

/** Convert hex color to DaisyUI OKLCH variable value (L% C H). */
export function hexToOklch(hex: string): string {
  const color = parse(hex)
  if (!color) return "60% 0.2 250"
  const o = oklch(color)
  if (!o) return "60% 0.2 250"
  const L = (o.l * 100).toFixed(2)
  const C = o.c.toFixed(3)
  const H = Number.isFinite(o.h ?? 0) ? (o.h ?? 0).toFixed(2) : "0"
  return `${L}% ${C} ${H}`
}

/** Convert hex color to shadcn/ui HSL variable value (H S% L%). */
export function hexToHsl(hex: string): string {
  const color = parse(hex)
  if (!color) return "217 91% 60%"
  const h = hsl(color)
  if (!h) return "217 91% 60%"
  const H = Number.isFinite(h.h ?? 0) ? Math.round(h.h ?? 0).toString() : "0"
  const S = `${(h.s * 100).toFixed(1)}%`
  const L = `${(h.l * 100).toFixed(1)}%`
  return `${H} ${S} ${L}`
}

/** Convert an OKLCH CSS string to a shadcn HSL CSS string. */
export function oklchStrToHsl(oklchStr: string): string | null {
  const color = parse(oklchStr)
  if (!color) return null
  const h = hsl(color)
  if (!h) return null
  const H = Number.isFinite(h.h ?? 0) ? Math.round(h.h ?? 0).toString() : "0"
  const S = `${(h.s * 100).toFixed(1)}%`
  const L = `${(h.l * 100).toFixed(1)}%`
  return `${H} ${S} ${L}`
}

/** Generate DaisyUI v4 CSS variable block for a custom theme. */
export function generateDaisyuiCss(colors: CustomThemeColors): string {
  const vars: Record<string, string> = {}
  if (colors.primary) vars["--color-primary"] = hexToOklch(colors.primary)
  if (colors.secondary) vars["--color-secondary"] = hexToOklch(colors.secondary)
  if (colors.accent) vars["--color-accent"] = hexToOklch(colors.accent)
  if (colors.neutral) vars["--color-neutral"] = hexToOklch(colors.neutral)
  if (colors["base-100"]) vars["--color-base-100"] = hexToOklch(colors["base-100"])
  if (colors.info) vars["--color-info"] = hexToOklch(colors.info)
  if (colors.success) vars["--color-success"] = hexToOklch(colors.success)
  if (colors.warning) vars["--color-warning"] = hexToOklch(colors.warning)
  if (colors.error) vars["--color-error"] = hexToOklch(colors.error)

  const entries = Object.entries(vars)
    .map(([k, v]) => `    ${k}: ${v};`)
    .join("\n")

  return `[data-theme="custom"] {\n${entries}\n}`
}

/** Generate shadcn/ui CSS variable block from the same colors. */
export function generateShadcnCss(colors: CustomThemeColors): string {
  const vars: Record<string, string> = {}
  if (colors.primary) {
    vars["--primary"] = hexToHsl(colors.primary)
    vars["--primary-foreground"] = "210 40% 98%"
  }
  if (colors.secondary) {
    vars["--secondary"] = hexToHsl(colors.secondary)
    vars["--secondary-foreground"] = "222.2 47.4% 11.2%"
  }
  if (colors.accent) {
    vars["--accent"] = hexToHsl(colors.accent)
    vars["--accent-foreground"] = "222.2 47.4% 11.2%"
  }
  if (colors["base-100"]) {
    vars["--background"] = hexToHsl(colors["base-100"])
    vars["--foreground"] = "222.2 84% 4.9%"
    vars["--card"] = hexToHsl(colors["base-100"])
    vars["--card-foreground"] = "222.2 84% 4.9%"
    vars["--popover"] = hexToHsl(colors["base-100"])
    vars["--popover-foreground"] = "222.2 84% 4.9%"
  }
  if (colors.neutral) {
    vars["--muted"] = hexToHsl(colors.neutral)
    vars["--muted-foreground"] = "215.4 16.3% 46.9%"
    vars["--border"] = hexToHsl(colors.neutral)
    vars["--input"] = hexToHsl(colors.neutral)
    vars["--ring"] = hexToHsl(colors.neutral)
  }
  if (colors.info) {
    // info maps to no direct shadcn var, skip
  }
  if (colors.success) {
    // no direct mapping
  }
  if (colors.warning) {
    // no direct mapping
  }
  if (colors.error) {
    vars["--destructive"] = hexToHsl(colors.error)
    vars["--destructive-foreground"] = "210 40% 98%"
  }

  const entries = Object.entries(vars)
    .map(([k, v]) => `    ${k}: ${v};`)
    .join("\n")

  return `:root {\n${entries}\n}`
}

/** Full list of DaisyUI built-in theme presets. */
export const DAISYUI_PRESETS = [
  "light",
  "dark",
  "cupcake",
  "bumblebee",
  "emerald",
  "corporate",
  "synthwave",
  "retro",
  "cyberpunk",
  "valentine",
  "halloween",
  "garden",
  "forest",
  "aqua",
  "lofi",
  "pastel",
  "fantasy",
  "wireframe",
  "black",
  "luxury",
  "dracula",
  "cmyk",
  "autumn",
  "business",
  "acid",
  "lemonade",
  "night",
  "coffee",
  "winter",
  "dim",
  "nord",
  "sunset",
  "caramellatte",
  "abyss",
  "silk",
] as const

export type DaisyuiPreset = (typeof DAISYUI_PRESETS)[number]

/** DaisyUI preset names that should enable dark-mode shadcn styling. */
export const DARK_PRESETS = [
  "dark",
  "night",
  "dracula",
  "black",
  "luxury",
  "business",
  "coffee",
  "dim",
  "sunset",
  "synthwave",
  "abyss",
]
