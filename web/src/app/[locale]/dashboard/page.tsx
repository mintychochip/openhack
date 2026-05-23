"use client"

import { useEffect, useState } from "react"
import { api } from "@/lib/api"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Users, Trophy, Mail, Award, Clock } from "lucide-react"
import { PhaseCountdown } from "@/components/CountdownTimer"
import { toast } from "@/hooks/use-toast"


export default function DashboardPage() {
  const [user, setUser] = useState<any>(null)
  const [stats, setStats] = useState({
    totalTeams: 0,
    totalProjects: 0,
    totalEvents: 0,
  })

  useEffect(() => {
    async function loadData() {
      try {
        const userData = await api.getMe()
        setUser(userData)

        // Load stats (these would be from analytics service in production)
        const [teams, projects, events] = await Promise.all([
          api.getTeams().catch(() => ({ teams: [] })),
          api.getProjects().catch(() => ({ projects: [] })),
          api.getEvents().catch(() => ({ events: [] })),
        ])

        setStats({
          totalTeams: teams.teams.length,
          totalProjects: projects.projects.length,
          totalEvents: events.events.length,
        })
      } catch (error: any) {
        console.error("Failed to load dashboard data:", error)
        toast({
          title: "Failed to load dashboard",
          description: error?.message || "Could not load your hackathon data. Please refresh.",
          variant: "destructive",
        })
      }
    }

    loadData()
  }, [])

  if (!user) {
    return <div className="flex items-center justify-center h-screen">Loading...</div>
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">Welcome back, {user.name}!</h1>
        <p className="text-muted-foreground">
          Here&apos;s what&apos;s happening with your hackathon
        </p>
      </div>

      {/* Phase Countdown */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="w-5 h-5" />
            Hackathon Phase
          </CardTitle>
          <CardDescription>
            Current phase countdown and timeline
          </CardDescription>
        </CardHeader>
        <CardContent>
          <PhaseCountdown hackathonId="00000000-0000-0000-0000-000000000001" />
        </CardContent>
      </Card>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Teams</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalTeams}</div>
            <p className="text-xs text-muted-foreground">
              Registered teams
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Projects</CardTitle>
            <Trophy className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalProjects}</div>
            <p className="text-xs text-muted-foreground">
              Submitted projects
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Events</CardTitle>
            <Mail className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalEvents}</div>
            <p className="text-xs text-muted-foreground">
              Scheduled events
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Your Rank</CardTitle>
            <Award className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">-</div>
            <p className="text-xs text-muted-foreground">
              Check leaderboard
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Create Team</CardTitle>
            <CardDescription>
              Form a team and start collaborating
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Teams can have up to 4 members. Create your team to start submitting projects.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Submit Project</CardTitle>
            <CardDescription>
              Submit your hackathon project
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Include description, repo URL, demo URL, and tags for your project.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>View Leaderboard</CardTitle>
            <CardDescription>
              See live rankings and vote
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Check real-time rankings and cast your public votes.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
