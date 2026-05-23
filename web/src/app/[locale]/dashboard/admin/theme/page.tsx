"use client"

import { useState, useCallback, useEffect, useRef } from "react"
import { useTheme, buildColorBridge, type FontConfig } from "@/contexts/theme-context"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { DARK_PRESETS, DAISYUI_PRESETS, generateDaisyuiCss, generateShadcnCss, type CustomThemeColors } from "@/lib/theme-colors"

const SEMANTIC_COLORS = [
  { key: "primary", label: "Primary" },
  { key: "secondary", label: "Secondary" },
  { key: "accent", label: "Accent" },
  { key: "neutral", label: "Neutral" },
  { key: "base-100", label: "Base Background" },
  { key: "info", label: "Info" },
  { key: "success", label: "Success" },
  { key: "warning", label: "Warning" },
  { key: "error", label: "Error" },
] as const

const FONT_PRESETS: { label: string; value: string; config: FontConfig }[] = [
  {
    label: "Modern Editorial",
    value: "modern-editorial",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Playfair+Display:wght@400;700;900&display=swap", family: "'Playfair Display', serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700&display=swap", family: "'Poppins', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap", family: "'Inter', sans-serif" },
    },
  },
  {
    label: "Clean Tech",
    value: "clean-tech",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;900&display=swap", family: "'Inter', sans-serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap", family: "'Inter', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap", family: "'Inter', sans-serif" },
    },
  },
  {
    label: "Academic Classic",
    value: "academic",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Merriweather:wght@400;700;900&display=swap", family: "'Merriweather', serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Lora:wght@400;500;600;700&display=swap", family: "'Lora', serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@400;500;600&display=swap", family: "'Source Sans 3', sans-serif" },
    },
  },
  {
    label: "Bold Startup",
    value: "bold-startup",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap", family: "'Space Grotesk', sans-serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Work+Sans:wght@400;500;600;700&display=swap", family: "'Work Sans', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap", family: "'Inter', sans-serif" },
    },
  },
  {
    label: "Creative Studio",
    value: "creative",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=DM+Serif+Display&display=swap", family: "'DM Serif Display', serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&display=swap", family: "'DM Sans', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600&display=swap", family: "'DM Sans', sans-serif" },
    },
  },
  {
    label: "Corporate",
    value: "corporate",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Roboto+Slab:wght@400;500;600;700&display=swap", family: "'Roboto Slab', serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;600;700&display=swap", family: "'Roboto', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;600&display=swap", family: "'Roboto', sans-serif" },
    },
  },
  {
    label: "Retro Terminal",
    value: "retro",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=VT323&display=swap", family: "'VT323', monospace" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap", family: "'Share Tech Mono', monospace" },
      body: { url: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap", family: "'JetBrains Mono', monospace" },
    },
  },
  {
    label: "Friendly Community",
    value: "friendly",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Nunito:wght@400;600;700;900&display=swap", family: "'Nunito', sans-serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Nunito:wght@400;600;700&display=swap", family: "'Nunito', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;500;600&display=swap", family: "'Open Sans', sans-serif" },
    },
  },
  {
    label: "Luxury",
    value: "luxury",
    config: {
      display: { url: "https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@400;500;600;700&display=swap", family: "'Cormorant Garamond', serif" },
      heading: { url: "https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700&display=swap", family: "'Montserrat', sans-serif" },
      body: { url: "https://fonts.googleapis.com/css2?family=Lato:wght@400;500;600&display=swap", family: "'Lato', sans-serif" },
    },
  },
  {
    label: "Custom",
    value: "custom",
    config: {},
  },
]

interface ExtractedBrand {
  logoUrl: string | null
  semanticColors: Record<string, string | null>
  suggestedPreset: string
  fontConfig: {
    display?: { url: string; family: string }
    heading?: { url: string; family: string }
    body?: { url: string; family: string }
  }
  extracted: {
    rawColors: string[]
    rawFonts: string[]
    websiteVibe: string
  }
}

function getContrastColor(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  return luminance > 0.5 ? "#000000" : "#ffffff"
}

export default function ThemeEditorPage() {
  const { user, isLoading } = useAuth()
  const { toast } = useToast()
  const { config, refresh } = useTheme()
  const [saving, setSaving] = useState(false)
  const [name, setName] = useState("OpenHack")
  const [tagline, setTagline] = useState("Build something amazing")
  const [logoUrl, setLogoUrl] = useState("")
  const [preset, setPreset] = useState("light")
  const [customColors, setCustomColors] = useState<CustomThemeColors>({})
  const [customCss, setCustomCss] = useState("")
  const [fontPreset, setFontPreset] = useState("custom")
  const [fontConfig, setFontConfig] = useState<FontConfig>({})

  // Brand extraction state
  const [brandUrl, setBrandUrl] = useState("")
  const [extracting, setExtracting] = useState(false)
  const [extracted, setExtracted] = useState<ExtractedBrand | null>(null)

  // Timer ref for color bridge setTimeout in global preview
  const previewTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Sync local state from config when it loads
  useEffect(() => {
    if (config) {
      setName(config.name)
      setTagline(config.tagline)
      setLogoUrl(config.logoUrl || "")
      setPreset(config.daisyuiPreset)
      setCustomColors(config.daisyuiCustomTheme || {})
      setCustomCss(config.customCss || "")
      setFontConfig(config.fontConfig || {})
    }
  }, [config])

  // Apply theme globally on the entire page in real-time (not just preview div)
  useEffect(() => {
    const html = document.documentElement

    // Clear any pending color bridge timer from previous change
    if (previewTimerRef.current) {
      clearTimeout(previewTimerRef.current)
      previewTimerRef.current = null
    }

    // Apply the selected preset to the entire page
    html.setAttribute("data-theme", preset)

    // Toggle dark class so shadcn/ui components also update
    const isDark = DARK_PRESETS.includes(preset as string)
    html.classList.toggle("dark", isDark)

    // Remove previously injected preview styles
    const removeIds = ["theme-preview-daisy", "theme-preview-shadcn", "theme-preview-bridge"]
    for (const id of removeIds) {
      document.getElementById(id)?.remove()
    }

    // Inject custom CSS for "custom" preset, or build color bridge for built-in presets
    if (preset === "custom" && Object.keys(customColors).length > 0) {
      const daisyStyle = document.createElement("style")
      daisyStyle.id = "theme-preview-daisy"
      daisyStyle.textContent = generateDaisyuiCss(customColors)
      document.head.appendChild(daisyStyle)

      const shadcnStyle = document.createElement("style")
      shadcnStyle.id = "theme-preview-shadcn"
      shadcnStyle.textContent = generateShadcnCss(customColors)
      document.head.appendChild(shadcnStyle)
    } else if (preset !== "custom") {
      // Build color bridge from daisyUI CSS vars -> shadcn HSL vars after a short
      // delay to let daisyUI apply its [data-theme] CSS variables first
      previewTimerRef.current = setTimeout(() => {
        previewTimerRef.current = null
        const bridgeStyle = document.createElement("style")
        bridgeStyle.id = "theme-preview-bridge"
        bridgeStyle.textContent = buildColorBridge()
        document.head.appendChild(bridgeStyle)
      }, 50)
    }

    return () => {
      if (previewTimerRef.current) {
        clearTimeout(previewTimerRef.current)
        previewTimerRef.current = null
      }
    }
  }, [preset, customColors])

  const updateColor = useCallback((key: string, value: string) => {
    setCustomColors((prev) => ({ ...prev, [key]: value }))
  }, [])

  const handleFontPresetChange = (value: string) => {
    setFontPreset(value)
    const preset = FONT_PRESETS.find((p) => p.value === value)
    if (preset && value !== "custom") {
      setFontConfig(preset.config)
    }
  }

  const updateFont = (role: "display" | "heading" | "body", field: "url" | "family", value: string) => {
    setFontConfig((prev) => ({
      ...prev,
      [role]: {
        ...prev[role],
        [field]: value,
      },
    }))
  }

  const handleSave = async (silent = false, presetOverride?: string) => {
    setSaving(true)
    const activePreset = presetOverride ?? preset
    try {
      const payload: Record<string, any> = {
        name,
        tagline,
        logo_url: logoUrl || null,
        daisyui_theme_preset: activePreset,
        daisyui_custom_theme: activePreset === "custom" ? customColors : {},
        custom_css: customCss,
        font_config: fontConfig,
      }

      await api.put("/api/core/info", payload)
      await refresh()
      if (!silent) {
        toast({ title: "Theme saved!", description: "Your changes are live." })
      }
    } catch (err: any) {
      if (!silent) {
        toast({
          title: "Failed to save",
          description: err.message || "Could not update theme",
          variant: "destructive",
        })
      }
    } finally {
      setSaving(false)
    }
  }

  const handlePresetChange = (newPreset: string) => {
    setPreset(newPreset)
    handleSave(true, newPreset)
  }

  const handleReset = () => {
    setName("OpenHack")
    setTagline("Build something amazing")
    setLogoUrl("")
    setPreset("light")
    setCustomColors({})
    setCustomCss("")
    setFontPreset("custom")
    setFontConfig({})
    setExtracted(null)
    toast({ title: "Reset to defaults", description: "Click Save to apply." })
  }

  const handleExtractBrand = async () => {
    if (!brandUrl.trim()) {
      toast({ title: "URL required", description: "Enter a website URL to extract from.", variant: "destructive" })
      return
    }
    setExtracting(true)
    setExtracted(null)
    try {
      const result = await api.extractBrand(brandUrl.trim())
      setExtracted(result)
      toast({ title: "Brand extracted!", description: `Detected preset: ${result.suggestedPreset}` })
    } catch (err: any) {
      toast({
        title: "Extraction failed",
        description: err.message || "Could not extract brand from that URL.",
        variant: "destructive",
      })
    } finally {
      setExtracting(false)
    }
  }

  const handleApplyExtracted = () => {
    if (!extracted) return

    if (extracted.logoUrl) {
      setLogoUrl(extracted.logoUrl)
    }

    const presetName = extracted.suggestedPreset
    if (DAISYUI_PRESETS.includes(presetName as any)) {
      setPreset(presetName)
    } else {
      setPreset("custom")
    }

    // Convert semantic_colors to customColors if using custom preset
    const colors: CustomThemeColors = {}
    for (const [key, value] of Object.entries(extracted.semanticColors)) {
      if (value) {
        colors[key as keyof CustomThemeColors] = value
      }
    }
    if (Object.keys(colors).length > 0) {
      setCustomColors(colors)
    }

    // Apply font config
    const newFonts: FontConfig = {}
    if (extracted.fontConfig.display) {
      newFonts.display = extracted.fontConfig.display
    }
    if (extracted.fontConfig.heading) {
      newFonts.heading = extracted.fontConfig.heading
    }
    if (extracted.fontConfig.body) {
      newFonts.body = extracted.fontConfig.body
    }
    if (Object.keys(newFonts).length > 0) {
      setFontConfig(newFonts)
      setFontPreset("custom")
    }

    toast({ title: "Applied!", description: "Review the preview and click Save Theme." })
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    )
  }

  if (!user || !user.roles.includes("admin")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  return (
    <div className="space-y-8 max-w-4xl">
      <div>
        <h1 className="text-3xl font-bold">Theme Editor</h1>
        <p className="text-muted-foreground">
          Customize colors, branding, typography, and CSS for your hackathon
        </p>
      </div>

      {/* Brand Extraction */}
      <Card>
        <CardHeader>
          <CardTitle>Extract Brand from Website</CardTitle>
          <CardDescription>
            Paste a company or event URL and we&apos;ll auto-detect colors, fonts, and logo
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input
              value={brandUrl}
              onChange={(e) => setBrandUrl(e.target.value)}
              placeholder="https://stripe.com"
              className="flex-1"
            />
            <Button onClick={handleExtractBrand} disabled={extracting}>
              {extracting ? "Extracting..." : "Extract Brand"}
            </Button>
          </div>

          {extracted && (
            <div className="space-y-4 pt-2">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {extracted.logoUrl && (
                  <div className="space-y-1">
                    <Label className="text-xs text-muted-foreground">Logo</Label>
                    <div className="p-3 border rounded-lg inline-block">
                      {/* eslint-disable-next-line @next/next/no-img-element */}
                      <img src={extracted.logoUrl} alt="Extracted logo" className="h-12 w-auto" />
                    </div>
                  </div>
                )}
                <div className="space-y-1">
                  <Label className="text-xs text-muted-foreground">Suggested Preset</Label>
                  <div className="text-sm font-medium capitalize">{extracted.suggestedPreset}</div>
                </div>
                <div className="space-y-1">
                  <Label className="text-xs text-muted-foreground">Detected Colors</Label>
                  <div className="flex gap-1 flex-wrap">
                    {extracted.extracted.rawColors.slice(0, 6).map((c) => (
                      <div
                        key={c}
                        className="w-6 h-6 rounded-full border"
                        style={{ backgroundColor: c }}
                        title={c}
                      />
                    ))}
                  </div>
                </div>
              </div>

              {Object.keys(extracted.semanticColors).length > 0 && (
                <div className="space-y-2">
                  <Label className="text-xs text-muted-foreground">Mapped Semantic Colors</Label>
                  <div className="flex gap-2 flex-wrap">
                    {SEMANTIC_COLORS.map(({ key, label }) => {
                      const color = extracted.semanticColors[key]
                      if (!color) return null
                      const textColor = getContrastColor(color)
                      return (
                        <div key={key} className="flex items-center gap-1.5 text-sm">
                          <div
                            className="w-6 h-6 rounded-md border shadow-sm flex items-center justify-center text-[8px] font-mono"
                            style={{ backgroundColor: color, color: textColor }}
                            title={color}
                          >
                            {color.slice(1, 4)}
                          </div>
                          <span className="capitalize">{label}</span>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              <Button onClick={handleApplyExtracted} variant="secondary">
                Apply Extracted Theme
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Branding */}
      <Card>
        <CardHeader>
          <CardTitle>Branding</CardTitle>
          <CardDescription>Hackathon name, tagline, and logo</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Hackathon Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Hackathon"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="tagline">Tagline</Label>
            <Input
              id="tagline"
              value={tagline}
              onChange={(e) => setTagline(e.target.value)}
              placeholder="Build something amazing"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="logo">Logo URL</Label>
            <Input
              id="logo"
              value={logoUrl}
              onChange={(e) => setLogoUrl(e.target.value)}
              placeholder="https://example.com/logo.png"
            />
            {logoUrl && (
              <div className="mt-2 p-4 border rounded-lg inline-block">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img src={logoUrl} alt="Logo preview" className="h-12 w-auto" />
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Typography */}
      <Card>
        <CardHeader>
          <CardTitle>Typography</CardTitle>
          <CardDescription>Choose fonts for display, headings, and body text</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Font Preset</Label>
            <select
              value={fontPreset}
              onChange={(e) => handleFontPresetChange(e.target.value)}
              className="w-full border rounded-md px-3 py-2 text-sm bg-background"
            >
              {FONT_PRESETS.map((p) => (
                <option key={p.value} value={p.value}>
                  {p.label}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground">
              Presets load curated Google Fonts. Select &quot;Custom&quot; to use your own CDN links.
            </p>
          </div>

          {fontPreset === "custom" && (
            <div className="space-y-6">
              {(["display", "heading", "body"] as const).map((role) => (
                <div key={role} className="space-y-2">
                  <Label className="capitalize">{role} Font</Label>
                  <Input
                    value={fontConfig[role]?.url || ""}
                    onChange={(e) => updateFont(role, "url", e.target.value)}
                    placeholder="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap"
                    className="mb-1"
                  />
                  <Input
                    value={fontConfig[role]?.family || ""}
                    onChange={(e) => updateFont(role, "family", e.target.value)}
                    placeholder="'Inter', sans-serif"
                  />
                </div>
              ))}
            </div>
          )}

          {fontPreset !== "custom" && (
            <div className="text-sm text-muted-foreground space-y-1">
              <p>
                <span className="font-medium">Display:</span> {fontConfig.display?.family}
              </p>
              <p>
                <span className="font-medium">Heading:</span> {fontConfig.heading?.family}
              </p>
              <p>
                <span className="font-medium">Body:</span> {fontConfig.body?.family}
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Theme Preset */}
      <Card>
        <CardHeader>
          <CardTitle>Theme Preset</CardTitle>
          <CardDescription>Choose a built-in DaisyUI theme</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {DAISYUI_PRESETS.map((p) => (
              <button
                key={p}
                onClick={() => handlePresetChange(p)}
                className={`px-3 py-2 rounded-lg border text-sm capitalize transition-colors ${
                  preset === p
                    ? "bg-primary text-primary-foreground border-primary"
                    : "bg-background hover:bg-secondary"
                }`}
              >
                {p}
              </button>
            ))}
            <button
              onClick={() => handlePresetChange("custom")}
              className={`px-3 py-2 rounded-lg border text-sm transition-colors ${
                preset === "custom"
                  ? "bg-primary text-primary-foreground border-primary"
                  : "bg-background hover:bg-secondary"
              }`}
            >
              Custom
            </button>
          </div>
        </CardContent>
      </Card>

      {/* Custom Colors */}
      {preset === "custom" && (
        <Card>
          <CardHeader>
            <CardTitle>Custom Colors</CardTitle>
            <CardDescription>Pick colors for each semantic role</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {SEMANTIC_COLORS.map(({ key, label }) => (
                <div key={key} className="flex items-center gap-3">
                  <input
                    type="color"
                    value={customColors[key] || "#3b82f6"}
                    onChange={(e) => updateColor(key, e.target.value)}
                    className="h-10 w-10 rounded cursor-pointer border-0 p-0"
                  />
                  <div className="flex-1">
                    <Label className="text-sm">{label}</Label>
                    <Input
                      value={customColors[key] || ""}
                      onChange={(e) => updateColor(key, e.target.value)}
                      placeholder="#3b82f6"
                      className="mt-1"
                    />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Live Preview */}
      <Card>
        <CardHeader>
          <CardTitle>Live Preview</CardTitle>
          <CardDescription>How your theme and typography look</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div
            className="p-6 rounded-xl border bg-base-100"
            data-theme={preset}
            style={{
              fontFamily: fontConfig.body?.family || "ui-sans-serif, system-ui, sans-serif",
            }}
          >
            <div className="space-y-4">
              <h3
                className="text-xl font-bold text-primary"
                style={{ fontFamily: fontConfig.display?.family || "inherit" }}
              >
                {name || "OpenHack"}
              </h3>
              <p className="text-muted-foreground">{tagline || "Build something amazing"}</p>
              <div className="flex gap-2 flex-wrap">
                <span className="badge badge-primary">Primary</span>
                <span className="badge badge-secondary">Secondary</span>
                <span className="badge badge-accent">Accent</span>
                <span className="badge badge-neutral">Neutral</span>
                <span className="badge badge-info">Info</span>
                <span className="badge badge-success">Success</span>
                <span className="badge badge-warning">Warning</span>
                <span className="badge badge-error">Error</span>
              </div>
              <div className="flex gap-2">
                <button className="btn btn-primary">Primary Button</button>
                <button className="btn btn-secondary">Secondary</button>
                <button className="btn btn-accent">Accent</button>
              </div>
              <div
                className="text-sm"
                style={{ fontFamily: fontConfig.heading?.family || "inherit" }}
              >
                <p className="font-semibold">Sample heading text</p>
                <p>Sample body text paragraph to preview your font choices.</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Custom CSS */}
      <Card>
        <CardHeader>
          <CardTitle>Custom CSS</CardTitle>
          <CardDescription>Override styles with your own CSS</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Textarea
            value={customCss}
            onChange={(e) => setCustomCss(e.target.value)}
            placeholder={`.hero-title { @apply text-5xl font-bold text-primary; }\n.custom-card { @apply card bg-base-100 shadow-xl; }`}
            rows={8}
            className="font-mono text-sm"
          />
          <p className="text-xs text-muted-foreground">
            You can use Tailwind classes with @apply or plain CSS. DaisyUI classes like{" "}
            <code>btn btn-primary</code>, <code>badge badge-secondary</code>, and{" "}
            <code>card bg-base-100</code> work here.
          </p>
        </CardContent>
      </Card>

      {/* Actions */}
      <div className="flex gap-4">
        <Button onClick={() => handleSave()} disabled={saving}>
          {saving ? "Saving..." : "Save All Changes"}
        </Button>
        <Button variant="outline" onClick={handleReset}>
          Reset to Defaults
        </Button>
      </div>
    </div>
  )
}
