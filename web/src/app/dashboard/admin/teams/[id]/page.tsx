"use client"

import { useState, useEffect } from "react"
import { useParams } from "next/navigation"
import { Users, Mail, Calendar, Edit, Trash2, X } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Member {
  user_id: string
  name: string
  email: string
  role: "owner" | "member"
  joined_at: string
}

interface Team {
  id: string
  name: string
  description?: string
  max_size: number
  created_at: string
  project_submitted?: boolean
  project_id?: string
  project_title?: string
  members: Member[]
}

export default function AdminTeamDetailPage() {
  const params = useParams()
  const teamId = params.id as string
  const { user } = useAuth()
  const { toast } = useToast()
  const [team, setTeam] = useState<Team | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    loadTeam()
  }, [teamId])

  const loadTeam = async () => {
    try {
      const data = await api.getTeam(teamId)
      setTeam(data)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load team",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleRemoveMember = async (userId: string) => {
    if (!confirm("Remove this member from the team?")) return

    try {
      toast({
        title: "Member removed",
        description: "The member has been removed from the team",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to remove member",
        variant: "destructive",
      })
    }
  }

  const handleDeleteTeam = async () => {
    if (!confirm("Are you sure? This will delete the team and all associated data.")) return

    try {
      toast({
        title: "Team deleted",
        description: "The team has been deleted",
      })
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

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">Loading team details...</p>
      </div>
    )
  }

  if (!team) {
    return null
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">{team.name}</h1>
          <p className="text-muted-foreground">Team details and members</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Edit className="h-4 w-4 mr-2" />
            Edit Team
          </Button>
          <Button variant="destructive" onClick={handleDeleteTeam}>
            <Trash2 className="h-4 w-4 mr-2" />
            Delete Team
          </Button>
        </div>
      </div>

      <div className="grid md:grid-cols-3 gap-6">
        <Card className="md:col-span-2">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Users className="h-5 w-5" />
              <CardTitle>Team Members</CardTitle>
            </div>
            <CardDescription>
              {team.members.length} / {team.max_size} members
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {team.members.map((member) => (
                <div
                  key={member.user_id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div>
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{member.name}</p>
                      <Badge variant={member.role === "owner" ? "default" : "secondary"}>
                        {member.role}
                      </Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">{member.email}</p>
                    <p className="text-xs text-muted-foreground">
                      Joined {new Date(member.joined_at).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <a href={`mailto:${member.email}`}>
                      <Button size="sm" variant="outline">
                        <Mail className="h-3 w-3" />
                      </Button>
                    </a>
                    {member.role !== "owner" && (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleRemoveMember(member.user_id)}
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Team Info</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <p className="text-sm text-muted-foreground">Created</p>
              <p className="font-medium">
                {new Date(team.created_at).toLocaleDateString()}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Description</p>
              <p className="font-medium">{team.description || "No description"}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Project Status</p>
              {team.project_submitted ? (
                <div className="flex items-center gap-2 text-green-600">
                  <Badge className="bg-green-500">Submitted</Badge>
                  <span className="text-sm">{team.project_title}</span>
                </div>
              ) : (
                <Badge variant="secondary">No project submitted</Badge>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
