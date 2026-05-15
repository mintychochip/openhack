"use client"

import { useState, useEffect } from "react"
import { Users, Search, Edit, Trash2, Eye } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"
import Link from "next/link"

interface Team {
  id: string
  name: string
  description?: string
  member_count: number
  max_size: number
  created_at: string
  project_submitted?: boolean
}

export default function AdminTeamsPage() {
  const { user } = useAuth()
  const [teams, setTeams] = useState<Team[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState("")
  const { toast } = useToast()

  useEffect(() => {
    loadTeams()
  }, [])

  const loadTeams = async () => {
    try {
      const data = await api.getTeams()
      setTeams(data.teams || [])
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load teams",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleDeleteTeam = async (teamId: string) => {
    if (!confirm("Are you sure? This will delete the team and all associated data.")) return

    try {
      // await api.adminDeleteTeam(teamId)
      toast({
        title: "Team deleted",
        description: "The team has been removed",
      })
      loadTeams()
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to delete team",
        variant: "destructive",
      })
    }
  }

  if (!user || user.role !== "admin") {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  const filteredTeams = teams.filter(
    (team) =>
      team.name.toLowerCase().includes(searchTerm.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Team Management</h1>
        <p className="text-muted-foreground">View and manage all teams</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Users className="h-5 w-5" />
              <CardTitle>All Teams</CardTitle>
            </div>
            <div className="relative w-64">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search teams..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading teams...</p>
            </div>
          ) : filteredTeams.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              {searchTerm ? "No teams found" : "No teams yet"}
            </div>
          ) : (
            <div className="space-y-2">
              {filteredTeams.map((team) => (
                <div
                  key={team.id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{team.name}</p>
                      {team.project_submitted && (
                        <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded">
                          Project Submitted
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {team.member_count}/{team.max_size} members • Created{" "}
                      {new Date(team.created_at).toLocaleDateString()}
                    </p>
                    {team.description && (
                      <p className="text-sm text-muted-foreground mt-1 line-clamp-1">
                        {team.description}
                      </p>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <Link href={`/dashboard/teams/${team.id}`}>
                      <Button size="sm" variant="outline">
                        <Eye className="h-3 w-3 mr-1" />
                        View
                      </Button>
                    </Link>
                    <Button size="sm" variant="outline">
                      <Edit className="h-3 w-3 mr-1" />
                      Edit
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={() => handleDeleteTeam(team.id)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
