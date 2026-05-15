"use client"

import Link from "next/link"
import { Home, Users, Trophy, BarChart3, Settings, LogOut, ClipboardList, Briefcase } from "lucide-react"
import { MobileNav } from "@/components/mobile-nav"
import { ThemeToggle } from "@/components/theme-toggle"
import { useAuth } from "@/contexts/auth-context"

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

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const { user, logout } = useAuth()

  let navItems = participantNavItems
  if (user?.role === "judge" || user?.role === "admin") {
    navItems = judgeNavItems
  } else if (user?.role === "sponsor") {
    navItems = sponsorNavItems
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <nav className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex">
              <div className="flex-shrink-0 flex items-center">
                <span className="text-2xl font-bold text-primary">🏆 OpenHack</span>
              </div>
              <MobileNav />
              <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
                {navItems.map((item) => (
                  <Link
                    key={item.href}
                    href={item.href}
                    className="border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700 inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium"
                  >
                    <item.icon className="w-4 h-4 mr-2" />
                    {item.label}
                  </Link>
                ))}
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <ThemeToggle />
              <Link
                href="/dashboard/settings"
                className="text-gray-500 hover:text-gray-700"
              >
                <Settings className="w-5 h-5" />
              </Link>
              {user?.avatarUrl ? (
                <img
                  src={user.avatarUrl}
                  alt={user.name}
                  className="w-8 h-8 rounded-full"
                />
              ) : (
                <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center text-white font-medium">
                  {user?.name?.charAt(0).toUpperCase() || "U"}
                </div>
              )}
              <button
                onClick={logout}
                className="text-gray-500 hover:text-destructive"
                title="Logout"
              >
                <LogOut className="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {children}
      </main>
    </div>
  )
}
