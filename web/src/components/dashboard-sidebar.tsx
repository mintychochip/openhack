"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"
import { LogOut, Settings } from "lucide-react"
import type { LucideIcon } from "lucide-react"

export interface NavItem {
  href: string
  icon: LucideIcon
  label: string
}

interface DashboardSidebarProps {
  brandName: string
  navItems: NavItem[]
  user: { name: string; avatarUrl?: string } | null
  logout: () => void
  className?: string
  showBrand?: boolean
}

export function DashboardSidebar({
  brandName,
  navItems,
  user,
  logout,
  className,
  showBrand = true,
}: DashboardSidebarProps) {
  const pathname = usePathname()

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {showBrand && (
        <div className="h-14 flex items-center px-4 border-b border-base-200 shrink-0">
          <span className="text-lg font-semibold tracking-tight text-base-content">
            {brandName}
          </span>
        </div>
      )}

      <nav className="flex-1 overflow-y-auto py-3 px-3 space-y-1">
        {navItems.map((item) => {
          const isActive =
            item.href === "/dashboard"
              ? pathname === "/dashboard"
              : pathname === item.href || pathname.startsWith(`${item.href}/`)

          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                isActive
                  ? "bg-base-200 text-base-content"
                  : "text-base-content/60 hover:text-base-content hover:bg-base-200/50"
              )}
            >
              <item.icon className="w-4 h-4 shrink-0" />
              {item.label}
            </Link>
          )
        })}
      </nav>

      <div className="p-3 border-t border-base-200 shrink-0 space-y-1">
        <Link
          href="/dashboard/settings"
          className={cn(
            "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors",
            pathname === "/dashboard/settings"
              ? "bg-base-200 text-base-content"
              : "text-base-content/60 hover:text-base-content hover:bg-base-200/50"
          )}
        >
          <Settings className="w-4 h-4 shrink-0" />
          Settings
        </Link>

        {user && (
          <div className="flex items-center gap-3 px-3 py-2">
            {user.avatarUrl ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={user.avatarUrl}
                alt={user.name}
                className="w-7 h-7 rounded-full object-cover"
              />
            ) : (
              <div className="w-7 h-7 rounded-full bg-primary flex items-center justify-center text-primary-foreground text-xs font-medium">
                {user.name?.charAt(0).toUpperCase() || "U"}
              </div>
            )}
            <span className="text-sm font-medium text-base-content truncate">
              {user.name}
            </span>
            <button
              onClick={logout}
              className="ml-auto text-base-content/50 hover:text-destructive transition-colors"
              title="Logout"
            >
              <LogOut className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
