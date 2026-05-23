"use client"

import * as React from "react"
import Link from "next/link"
import { usePathname } from "next/navigation"
import { Menu, X, ClipboardList } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { ThemeToggle } from "@/components/theme-toggle"
import { useAuth } from "@/contexts/auth-context"

interface NavItem {
  href: string
  label: string
}

const participantNavItems: NavItem[] = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/dashboard/teams", label: "Teams" },
  { href: "/dashboard/projects", label: "Projects" },
  { href: "/dashboard/events", label: "Events" },
  { href: "/dashboard/leaderboard", label: "Leaderboard" },
]

const judgeNavItems: NavItem[] = [
  ...participantNavItems,
  { href: "/dashboard/judging", label: "Judging" },
]

const sponsorNavItems: NavItem[] = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/dashboard/sponsor/booth", label: "My Booth" },
  { href: "/dashboard/sponsor/prizes", label: "Prizes" },
  { href: "/dashboard/sponsor/submissions", label: "Submissions" },
]

const adminNavItems: NavItem[] = [
  ...judgeNavItems,
  { href: "/dashboard/admin/hackathon", label: "Hackathon" },
  { href: "/dashboard/admin/theme", label: "Theme" },
]

export function MobileNav() {
  const [isOpen, setIsOpen] = React.useState(false)
  const pathname = usePathname()
  const { user } = useAuth()

  React.useEffect(() => {
    setIsOpen(false)
  }, [pathname])

  let navItems = participantNavItems
  if (user?.roles.includes("admin")) {
    navItems = adminNavItems
  } else if (user?.roles.includes("judge")) {
    navItems = judgeNavItems
  } else if (user?.roles.includes("sponsor")) {
    navItems = sponsorNavItems
  }

  return (
    <>
      <Button
        variant="ghost"
        size="icon"
        onClick={() => setIsOpen(!isOpen)}
        className="sm:hidden"
      >
        {isOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
      </Button>

      <div
        className={cn(
          "fixed inset-0 top-16 z-50 bg-background sm:hidden",
          isOpen ? "block" : "hidden"
        )}
      >
        <nav className="container mx-auto px-4 py-6 flex flex-col gap-2">
          {navItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "px-4 py-3 rounded-lg text-sm font-medium transition-colors",
                pathname === item.href
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-secondary"
              )}
            >
              {item.label}
            </Link>
          ))}
          <div className="border-t mt-4 pt-4 flex items-center justify-between">
            <Link
              href="/dashboard/settings"
              className="px-4 py-3 rounded-lg text-sm font-medium hover:bg-secondary transition-colors"
            >
              Settings
            </Link>
            <ThemeToggle />
          </div>
        </nav>
      </div>
    </>
  )
}
