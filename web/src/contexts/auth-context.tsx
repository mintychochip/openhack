"use client"

import React, { createContext, useContext, useEffect, useState } from "react"
import { api } from "@/lib/api"
import { useRouter } from "next/navigation"
import { toast } from "@/hooks/use-toast"

interface User {
  id: string
  email: string
  name: string
  roles: string[]
  avatarUrl?: string
  githubUsername?: string
  mfaEnabled: boolean
}

interface AuthContextType {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (email: string, password: string) => Promise<void>
  logout: () => void
  updateUser: (user: Partial<User>) => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()

  useEffect(() => {
    const authRoutes = ["/login", "/register", "/forgot-password", "/reset-password"]
    if (typeof window !== "undefined" && authRoutes.includes(window.location.pathname)) {
      setIsLoading(false)
      return
    }

    async function loadUser() {
      try {
        const userData = await api.getMe()
        setUser(userData)
      } catch (error: any) {
        api.logout()
        setUser(null)
        toast({
          title: "Session expired",
          description: error?.message || "Please log in again.",
          variant: "destructive",
        })
      } finally {
        setIsLoading(false)
      }
    }

    loadUser()
  }, [])

  const login = async (email: string, password: string) => {
    await api.login(email, password)
    const userData = await api.getMe()
    setUser(userData)
  }

  const logout = () => {
    api.logout()
    setUser(null)
    router.push("/login")
  }

  const updateUser = (userData: Partial<User>) => {
    if (user) {
      setUser({ ...user, ...userData })
    }
  }

  const value: AuthContextType = {
    user,
    isLoading,
    isAuthenticated: !!user,
    login,
    logout,
    updateUser,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return context
}
