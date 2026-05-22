"use client"

import { useState, useEffect, useCallback } from "react"
import { useParams, useRouter } from "next/navigation"
import { Star, Save, AlertCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { Slider } from "@/components/ui/slider"

interface Criterion {
  id: string
  name: string
  description?: string
  weight: number
  max_score: number
}

interface Rubric {
  id: string
  name: string
  description?: string
  criteria: Criterion[]
}

interface Assignment {
  id: string
  project_id: string
  project_title: string
  team_name: string
  rubric_id: string
  rubric: Rubric
  phase_name: string
}

export default function ScoreProjectPage() {
  const params = useParams()
  const router = useRouter()
  const assignmentId = params.assignmentId as string
  const { toast } = useToast()

  const [assignment, setAssignment] = useState<Assignment | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [scores, setScores] = useState<Record<string, number>>({})
  const [comment, setComment] = useState("")

  const loadAssignment = useCallback(async () => {
    try {
      const data = await api.getMyAssignments()
      const assignmentData = data.assignments.find((a: Assignment) => a.id === assignmentId)
      if (!assignmentData) {
        toast({
          title: "Not found",
          description: "Assignment not found",
          variant: "destructive",
        })
        router.push("/dashboard/judging")
        return
      }
      setAssignment(assignmentData)
      // Initialize scores with mid-point values
      const initialScores: Record<string, number> = {}
      assignmentData.rubric.criteria.forEach((criterion: Criterion) => {
        initialScores[criterion.id] = Math.round(criterion.max_score / 2)
      })
      setScores(initialScores)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load assignment",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }, [assignmentId, toast, router])

  useEffect(() => {
    loadAssignment()
  }, [loadAssignment])

  const handleScoreChange = (criterionId: string, value: number) => {
    setScores((prev) => ({ ...prev, [criterionId]: value }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)

    try {
      await api.submitScore(assignmentId, scores, comment || undefined)
      toast({
        title: "Score submitted",
        description: "Your evaluation has been recorded",
      })
      router.push("/dashboard/judging")
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to submit score",
        variant: "destructive",
      })
    } finally {
      setIsSubmitting(false)
    }
  }

  const calculateTotalScore = () => {
    if (!assignment) return { total: 0, maxTotal: 0, percentage: 0 }
    let total = 0
    let maxTotal = 0
    assignment.rubric.criteria.forEach((criterion) => {
      total += scores[criterion.id] || 0
      maxTotal += criterion.max_score
    })
    return { total, maxTotal, percentage: maxTotal > 0 ? (total / maxTotal) * 100 : 0 }
  }

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">Loading assignment...</p>
      </div>
    )
  }

  if (!assignment) {
    return null
  }

  const { total, maxTotal, percentage } = calculateTotalScore()

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Score Project</h1>
        <p className="text-muted-foreground">Evaluate the project using the rubric</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{assignment.project_title}</CardTitle>
          <CardDescription>
            Team: {assignment.team_name} • Phase: {assignment.phase_name}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="mb-4">
            <h3 className="font-semibold mb-2">Rubric: {assignment.rubric.name}</h3>
            <p className="text-sm text-muted-foreground">
              {assignment.rubric.description || "No description provided"}
            </p>
          </div>
          <div className="bg-secondary/50 rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Current Score</span>
              <span className="text-lg font-bold">
                {total} / {maxTotal} ({percentage.toFixed(1)}%)
              </span>
            </div>
            <div className="w-full bg-secondary rounded-full h-2">
              <div
                className="bg-primary h-2 rounded-full transition-all"
                style={{ width: `${percentage}%` }}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <form onSubmit={handleSubmit} className="space-y-6">
        {assignment.rubric.criteria.map((criterion, index) => (
          <Card key={criterion.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-lg">
                    {index + 1}. {criterion.name}
                  </CardTitle>
                  {criterion.description && (
                    <CardDescription>{criterion.description}</CardDescription>
                  )}
                </div>
                <div className="text-right">
                  <span className="text-sm text-muted-foreground">Weight: </span>
                  <span className="font-medium">{criterion.weight}x</span>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label>Score</Label>
                  <span className="text-lg font-bold">
                    {scores[criterion.id] || 0} / {criterion.max_score}
                  </span>
                </div>
                <Slider
                  value={[scores[criterion.id] || 0]}
                  onValueChange={(values) => handleScoreChange(criterion.id, values[0])}
                  max={criterion.max_score}
                  step={1}
                  className="py-4"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>0</span>
                  <span>{criterion.max_score}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}

        <Card>
          <CardHeader>
            <CardTitle>Additional Comments</CardTitle>
            <CardDescription>
              Optional feedback for the team
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Textarea
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              placeholder="Share your feedback, suggestions, or highlights..."
              rows={4}
              className="resize-none"
            />
          </CardContent>
        </Card>

        <div className="flex gap-2">
          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? (
              <>
                <Save className="h-4 w-4 mr-2 animate-spin" />
                Submitting...
              </>
            ) : (
              <>
                <Star className="h-4 w-4 mr-2" />
                Submit Score
              </>
            )}
          </Button>
          <Button type="button" variant="outline" onClick={() => router.back()}>
            Cancel
          </Button>
        </div>

        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Once submitted, scores can only be modified by administrators. Please review your scores before submitting.
          </AlertDescription>
        </Alert>
      </form>
    </div>
  )
}
