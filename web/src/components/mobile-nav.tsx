"use client"

import * as React from "react"
import * as Dialog from "@radix-ui/react-dialog"
import { Menu, X } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { DashboardSidebar, type NavItem } from "@/components/dashboard-sidebar"

interface MobileNavProps {
  brandName: string
  navItems: NavItem[]
  user: { name: string; avatarUrl?: string } | null
  logout: () => void
}

export function MobileNav({ brandName, navItems, user, logout }: MobileNavProps) {
  const [open, setOpen] = React.useState(false)

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <Button variant="ghost" size="icon" className="md:hidden">
          <Menu className="h-5 w-5" />
        </Button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            "fixed inset-0 z-50 bg-black/40",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0"
          )}
        />
        <Dialog.Content
          className={cn(
            "fixed inset-y-0 left-0 z-50 w-72 bg-base-100 border-r border-base-200 shadow-xl outline-none",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=open]:slide-in-from-left data-[state=closed]:slide-out-to-left",
            "duration-300"
          )}
        >
          <Dialog.Title className="sr-only">Navigation Menu</Dialog.Title>
          <Dialog.Description className="sr-only">
            Mobile navigation menu for the dashboard.
          </Dialog.Description>

          <div className="flex flex-col h-full">
            <div className="h-14 flex items-center justify-between px-4 border-b border-base-200 shrink-0">
              <span className="text-lg font-semibold tracking-tight text-base-content">
                {brandName}
              </span>
              <Dialog.Close asChild>
                <Button variant="ghost" size="icon">
                  <X className="h-5 w-5" />
                </Button>
              </Dialog.Close>
            </div>

            <DashboardSidebar
              brandName={brandName}
              navItems={navItems}
              user={user}
              logout={() => {
                setOpen(false)
                logout()
              }}
              showBrand={false}
              className="flex-1"
            />
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
