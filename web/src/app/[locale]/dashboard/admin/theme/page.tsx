"use client"

import { useState, useCallback } from "react"
import { useTheme, type FontConfig } from "@/contexts/theme-context"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { DAISYUI_PRESETS } from "@/lib/theme-colors"

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

export default function ThemeEditorPage() {
  const { user } = useAuth()
  const { config, refresh } = useTheme()
  const { toast } = useToast()

  const [name, setName] = useState(config?.name || "OpenHack")
  const [tagline, setTagline] = useState(config?.tagline || "")
  const [logoUrl, setLogoUrl] = useState(config?.logoUrl || "")
  const [preset, setPreset] = useState(config?.daisyuiPreset || "light")
  const [customColors, setCustomColors] = useState<Record<string, string>>(
    (config?.daisyuiCustomTheme as Record<string, string>) || {}
  )
  const [customCss, setCustomCss] = useState(config?.customCss || "")
  const [fontPreset, setFontPreset] = useState("custom")
  const [fontConfig, setFontConfig] = useState<FontConfig>(config?.fontConfig || {})
  const [saving, setSaving] = useState(false)

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

  const handleSave = async () => {
    setSaving(true)
    try {
      const payload: Record<string, any> = {
        name,
        tagline,
        logo_url: logoUrl || null,
        daisyui_theme_preset: preset,
        daisyui_custom_theme: preset === "custom" ? customColors : {},
        custom_css: customCss,
        font_config: fontConfig,
      }

      await api.put("/api/core/info", payload)
      await refresh()
      toast({ title: "Theme saved!", description: "Your changes are live." })
    } catch (err: any) {
      toast({
        title: "Failed to save",
        description: err.message || "Could not update theme",
        variant: "destructive",
      })
    } finally {
      setSaving(false)
    }
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
    toast({ title: "Reset to defaults", description: "Click Save to apply." })
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
                onClick={() => setPreset(p)}
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
              onClick={() => setPreset("custom")}
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
        <Button onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save Theme"}
        </Button>
        <Button variant="outline" onClick={handleReset}>
          Reset to Defaults
        </Button>
      </div>
    </div>
  )
}
