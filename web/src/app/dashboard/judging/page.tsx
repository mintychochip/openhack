"use client"

import { useState, useEffect } from "react"
import Link from "next/link"
import { ClipboardList, CheckCircle, Clock, TrendingUp } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Assignment {
  id: string
  project_id: string
  project_title: string
  team_name: string
  rubric_id: string
  rubric_name: string
  phase_id: string
  phase_name: string
  status: "pending" | "in_progress" | "completed"
  score_submitted?: boolean
  created_at: string
  deadline?: string
}

interface DashboardStats {
  total: number
  pending: number
  in_progress: number
  completed: number
}

export default function JudgeDashboardPage() {
  const { user } = useAuth()
  const [assignments, setAssignments] = useState<Assignment[]>([])
  const [stats, setStats] = useState<DashboardStats>({ total: 0, pending: 0, in_progress: 0, completed: 0 })
  const [isLoading, setIsLoading] = useState(true)
  const { toast } = useToast()

  useEffect(() => {
    loadAssignments()
  }, [])

  const loadAssignments = async () => {
    try {
      const data = await api.getMyAssignments()
      const assignmentsList = data.assignments || []
      setAssignments(assignmentsList)

      const stats = {
        total: assignmentsList.length,
        pending: assignmentsList.filter((a: Assignment) => a.status === "pending").length,
        in_progress: assignmentsList.filter((a: Assignment) => a.status === "in_progress").length,
        completed: assignmentsList.filter((a: Assignment) => a.status === "completed").length,
      }
      setStats(stats)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load assignments",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case "pending":
        return "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
      case "in_progress":
        return "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
      case "completed":
        return "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
      default:
        return "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200"
    }
  }

  if (!user || (user.role !== "judge" && user.role !== "admin")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">You don't have permission to access the judging interface</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Judge Dashboard</h1>
        <p className="text-muted-foreground">Manage your judging assignments</p>
      </div>

      <div className="grid md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total</CardTitle>
            <ClipboardList className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pending</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.pending}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">In Progress</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.in_progress}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Completed</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.completed}</div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Your Assignments</CardTitle>
          <CardDescription>
            Projects assigned to you for judging
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading assignments...</p>
            </div>
          ) : assignments.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              No assignments yet. You'll be notified when projects are assigned to you.
            </div>
          ) : (
            <div className="space-y-4">
              {assignments.map((assignment) => (
                <div
                  key={assignment.id}
                  className="flex items-center justify-between p-4 border rounded-lg hover:bg-secondary/50 transition-colors"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="font-semibold">{assignment.project_title}</h3>
                      <span className={`text-xs px-2 py-1 rounded ${getStatusColor(assignment.status)}`}>
                        {assignment.status}
                      </span>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <span>{assignment.team_name}</span>
                      <span>•</span>
                      <span>{assignment.rubric_name}</span>
                      <span>•</span>
                      <span>{assignment.phase_name}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    {assignment.deadline && (
                      <div className="text-sm text-muted-foreground">
                        Due: {new Date(assignment.deadline).toLocaleDateString()}
                      </div>
                    )}
                    <Link href={`/dashboard/judging/score/${assignment.id}`}>
                      <Button
                        size="sm"
                        variant={assignment.status === "completed" ? "outline" : "default"}
                        disabled={assignment.status === "completed"}
                      >
                        {assignment.status === "completed" ? "View Score" : "Score"}
                      </Button>
                    </Link>
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
