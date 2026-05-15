"use client"

import { useState, useEffect } from "react"
import { useParams } from "next/navigation"
import { BookOpen, Weight, Percent } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"

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
  versions?: number
}

export default function RubricViewerPage() {
  const params = useParams()
  const rubricId = params.id as string
  const { toast } = useToast()
  const [rubric, setRubric] = useState<Rubric | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    loadRubric()
  }, [rubricId])

  const loadRubric = async () => {
    try {
      const data = await api.getRubrics()
      const rubricData = data.rubrics.find((r: Rubric) => r.id === rubricId)
      if (!rubricData) {
        toast({
          title: "Not found",
          description: "Rubric not found",
          variant: "destructive",
        })
        return
      }
      setRubric(rubricData)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load rubric",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  if (isLoading) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">Loading rubric...</p>
      </div>
    )
  }

  if (!rubric) {
    return null
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Rubric</h1>
        <p className="text-muted-foreground">Evaluation criteria and scoring guide</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            <CardTitle>{rubric.name}</CardTitle>
          </div>
          <CardDescription>
            {rubric.description || "No description provided"}
          </CardDescription>
          {rubric.versions && (
            <p className="text-xs text-muted-foreground mt-2">
              Version {rubric.versions}
            </p>
          )}
        </CardHeader>
      </Card>

      <div className="space-y-4">
        {rubric.criteria.map((criterion, index) => (
          <Card key={criterion.id}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">
                  {index + 1}. {criterion.name}
                </CardTitle>
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-1 text-sm text-muted-foreground">
                    <Weight className="h-4 w-4" />
                    <span>Weight: {criterion.weight}x</span>
                  </div>
                  <div className="flex items-center gap-1 text-sm text-muted-foreground">
                    <Percent className="h-4 w-4" />
                    <span>Max: {criterion.max_score} pts</span>
                  </div>
                </div>
              </div>
              {criterion.description && (
                <CardDescription>{criterion.description}</CardDescription>
              )}
            </CardHeader>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Scoring Guide</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between py-2 border-b">
              <span>Total Criteria:</span>
              <span className="font-medium">{rubric.criteria.length}</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span>Total Weight:</span>
              <span className="font-medium">
                {rubric.criteria.reduce((sum, c) => sum + c.weight, 0)}x
              </span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span>Maximum Score:</span>
              <span className="font-medium">
                {rubric.criteria.reduce((sum, c) => sum + c.max_score, 0)} points
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
