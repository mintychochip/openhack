"use client"

import { useState, useEffect } from "react"
import { History, TrendingUp } from "lucide-react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface ScoreHistory {
  id: string
  assignment_id: string
  project_title: string
  team_name: string
  phase_name: string
  total_score: number
  normalized_score?: number
  submitted_at: string
}

export default function ScoreHistoryPage() {
  const { user } = useAuth()
  const [scores, setScores] = useState<ScoreHistory[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const { toast } = useToast()

  useEffect(() => {
    loadHistory()
  }, [])

  const loadHistory = async () => {
    try {
      // This would need a new API endpoint: /api/judging/scores/me
      // For now, we'll use assignments and filter completed ones
      const assignmentsData = await api.getMyAssignments()
      const completedAssignments = assignmentsData.assignments.filter(
        (a: any) => a.status === "completed"
      )
      
      // Transform to score history format
      const scoreHistory = completedAssignments.map((assignment: any) => ({
        id: assignment.id,
        assignment_id: assignment.id,
        project_title: assignment.project_title,
        team_name: assignment.team_name,
        phase_name: assignment.phase_name,
        total_score: assignment.total_score || 0,
        normalized_score: assignment.normalized_score,
        submitted_at: assignment.submitted_at || assignment.updated_at,
      }))
      
      setScores(scoreHistory)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load score history",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  if (!user || (user.role !== "judge" && user.role !== "admin")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">You don't have permission to access this page</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Score History</h1>
        <p className="text-muted-foreground">Your past evaluations</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <History className="h-5 w-5" />
            <CardTitle>Submitted Scores</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading history...</p>
            </div>
          ) : scores.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              No scores submitted yet
            </div>
          ) : (
            <div className="space-y-4">
              {scores.map((score) => (
                <div
                  key={score.id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div>
                    <h3 className="font-semibold">{score.project_title}</h3>
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <span>{score.team_name}</span>
                      <span>•</span>
                      <span>{score.phase_name}</span>
                      <span>•</span>
                      <span>
                        {new Date(score.submitted_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="flex items-center gap-2">
                      <TrendingUp className="h-4 w-4 text-muted-foreground" />
                      <span className="text-2xl font-bold">{score.total_score.toFixed(1)}</span>
                    </div>
                    {score.normalized_score !== undefined && (
                      <p className="text-xs text-muted-foreground">
                        Normalized: {score.normalized_score.toFixed(2)}
                      </p>
                    )}
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
