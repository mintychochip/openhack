"use client"

import * as React from "react"
import { Moon, Sun } from "lucide-react"
import { Button } from "@/components/ui/button"
import { useTheme } from "@/contexts/theme-context"
import { DARK_PRESETS } from "@/lib/theme-colors"

export function ThemeToggle() {
  const { config } = useTheme()
  const [theme, setTheme] = React.useState<"light" | "dark">("light")
  const initializedRef = React.useRef(false)

  React.useEffect(() => {
    if (!config || initializedRef.current) return
    const saved = localStorage.getItem("theme") as "light" | "dark" | null
    const presetIsDark = DARK_PRESETS.includes(config.daisyuiPreset as string)
    const initial = saved || (presetIsDark ? "dark" : "light")
    setTheme(initial)
    initializedRef.current = true
  }, [config])

  const toggleTheme = () => {
    const newTheme = theme === "light" ? "dark" : "light"
    setTheme(newTheme)
    localStorage.setItem("theme", newTheme)
    document.documentElement.classList.toggle("dark", newTheme === "dark")
  }

  return (
    <Button variant="ghost" size="icon" onClick={toggleTheme} className="relative">
      <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
      <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
      <span className="sr-only">Toggle theme</span>
    </Button>
  )
}
