"use client"

import { useState, useEffect } from "react"
import {
  Home,
  Users,
  Trophy,
  BarChart3,
  ClipboardList,
  Briefcase,
  Palette,
  Settings,
} from "lucide-react"
import { MobileNav } from "@/components/mobile-nav"
import { ThemeToggle } from "@/components/theme-toggle"
import { useAuth } from "@/contexts/auth-context"
import { useTheme } from "@/contexts/theme-context"
import { DashboardSidebar } from "@/components/dashboard-sidebar"

const participantNavItems = [
  { href: "/dashboard", icon: Home, label: "Dashboard" },
  { href: "/dashboard/teams", icon: Users, label: "Teams" },
  { href: "/dashboard/projects", icon: Trophy, label: "Projects" },
  { href: "/dashboard/events", icon: BarChart3, label: "Events" },
  { href: "/dashboard/leaderboard", icon: ClipboardList, label: "Leaderboard" },
]

const judgeNavItems = [
  ...participantNavItems,
  { href: "/dashboard/judging", icon: ClipboardList, label: "Judging" },
]

const sponsorNavItems = [
  { href: "/dashboard", icon: Home, label: "Dashboard" },
  { href: "/dashboard/sponsor/booth", icon: Briefcase, label: "Booth" },
  { href: "/dashboard/sponsor/prizes", icon: Trophy, label: "Prizes" },
  { href: "/dashboard/sponsor/submissions", icon: ClipboardList, label: "Submissions" },
]

const adminNavItems = [
  { href: "/dashboard", icon: Home, label: "Dashboard" },
  { href: "/dashboard/teams", icon: Users, label: "Teams" },
  { href: "/dashboard/projects", icon: Trophy, label: "Projects" },
  { href: "/dashboard/events", icon: BarChart3, label: "Events" },
  { href: "/dashboard/leaderboard", icon: ClipboardList, label: "Leaderboard" },
  { href: "/dashboard/judging", icon: ClipboardList, label: "Judging" },
  { href: "/dashboard/admin/hackathon", icon: Settings, label: "Hackathon" },
  { href: "/dashboard/admin/theme", icon: Palette, label: "Theme" },
]

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const { user, logout } = useAuth()
  const { config } = useTheme()
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
  }, [])

  const brandName = mounted ? config?.name || "OpenHack" : "OpenHack"

  let navItems = participantNavItems
  if (user?.roles.includes("admin")) {
    navItems = adminNavItems
  } else if (user?.roles.includes("judge")) {
    navItems = judgeNavItems
  } else if (user?.roles.includes("sponsor")) {
    navItems = sponsorNavItems
  }

  return (
    <div className="min-h-screen bg-base-100 flex">
      {/* Desktop Sidebar */}
      <aside className="hidden md:flex fixed left-0 top-0 z-40 w-64 h-screen border-r border-base-200 bg-base-100 flex-col">
        <DashboardSidebar
          brandName={brandName}
          navItems={navItems}
          user={user}
          logout={logout}
        />
      </aside>

      {/* Main area */}
      <div className="flex-1 flex flex-col md:ml-64 min-h-screen">
        {/* Top Header */}
        <header className="sticky top-0 z-30 h-14 flex items-center justify-between px-4 sm:px-6 border-b border-base-200 bg-base-100/80 backdrop-blur-md">
          <div className="flex items-center gap-3">
            <MobileNav brandName={brandName} navItems={navItems} user={user} logout={logout} />
            <span className="text-sm font-medium text-base-content md:hidden">
              {brandName}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <ThemeToggle />
            {user?.avatarUrl ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={user.avatarUrl}
                alt={user.name}
                className="w-7 h-7 rounded-full object-cover"
              />
            ) : (
              <div className="w-7 h-7 rounded-full bg-primary flex items-center justify-center text-primary-foreground text-xs font-medium">
                {user?.name?.charAt(0).toUpperCase() || "U"}
              </div>
            )}
          </div>
        </header>

        {/* Content */}
        <main className="flex-1 bg-base-200/30 p-4 sm:p-6 lg:p-8 overflow-y-auto">
          {children}
        </main>
      </div>
    </div>
  )
}
