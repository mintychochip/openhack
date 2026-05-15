"use client"

import { useState, useEffect } from "react"
import { Settings, Plus, Edit, Trash2, Play, Pause, Square } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Rubric {
  id: string
  name: string
  description?: string
  criteria_count: number
  is_active: boolean
  created_at: string
}

interface Phase {
  id: string
  name: string
  description?: string
  start_time: string
  end_time?: string
  is_active: boolean
  order: number
}

export default function AdminJudgingPage() {
  const { user } = useAuth()
  const [activeTab, setActiveTab] = useState<"rubrics" | "phases">("rubrics")
  const [rubrics, setRubrics] = useState<Rubric[]>([])
  const [phases, setPhases] = useState<Phase[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [showRubricForm, setShowRubricForm] = useState(false)
  const [showPhaseForm, setShowPhaseForm] = useState(false)
  const { toast } = useToast()

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      const [rubricsData, phasesData] = await Promise.all([
        api.getRubrics(),
        // api.getPhases() - not in API client yet
        Promise.resolve({ phases: [] }),
      ])
      setRubrics(rubricsData.rubrics || [])
      setPhases(phasesData.phases || [])
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load judging config",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
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

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Judging Configuration</h1>
          <p className="text-muted-foreground">Manage rubrics and judging phases</p>
        </div>
      </div>

      <div className="flex gap-2 border-b">
        <button
          onClick={() => setActiveTab("rubrics")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "rubrics"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          Rubrics
        </button>
        <button
          onClick={() => setActiveTab("phases")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "phases"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          Phases
        </button>
      </div>

      {activeTab === "rubrics" && (
        <div className="space-y-4">
          <div className="flex justify-between">
            <div>
              <h2 className="text-xl font-semibold">Evaluation Rubrics</h2>
              <p className="text-sm text-muted-foreground">
                Define criteria for judging projects
              </p>
            </div>
            <Button onClick={() => setShowRubricForm(!showRubricForm)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Rubric
            </Button>
          </div>

          {showRubricForm && (
            <Card>
              <CardHeader>
                <CardTitle>New Rubric</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="name">Rubric Name</Label>
                  <Input id="name" placeholder="e.g., Default Judging Rubric" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="description">Description</Label>
                  <Textarea
                    id="description"
                    placeholder="Describe what this rubric evaluates..."
                  />
                </div>
                <div className="space-y-2">
                  <Label>Criteria</Label>
                  <p className="text-sm text-muted-foreground">
                    Criteria will be added in the next step
                  </p>
                </div>
                <div className="flex gap-2">
                  <Button>Save Rubric</Button>
                  <Button
                    variant="outline"
                    onClick={() => setShowRubricForm(false)}
                  >
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {rubrics.length === 0 ? (
            <Card>
              <CardContent className="py-12 text-center text-muted-foreground">
                No rubrics yet. Create your first judging rubric.
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4">
              {rubrics.map((rubric) => (
                <Card key={rubric.id}>
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div>
                        <CardTitle>{rubric.name}</CardTitle>
                        <CardDescription>
                          {rubric.description || "No description"}
                        </CardDescription>
                      </div>
                      <span
                        className={`text-xs px-2 py-1 rounded ${
                          rubric.is_active
                            ? "bg-green-100 text-green-800"
                            : "bg-gray-100 text-gray-800"
                        }`}
                      >
                        {rubric.is_active ? "Active" : "Inactive"}
                      </span>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="flex items-center justify-between">
                      <div className="text-sm text-muted-foreground">
                        {rubric.criteria_count} criteria • Created{" "}
                        {new Date(rubric.created_at).toLocaleDateString()}
                      </div>
                      <div className="flex gap-2">
                        <Button size="sm" variant="outline">
                          <Edit className="h-3 w-3 mr-1" />
                          Edit
                        </Button>
                        <Button size="sm" variant="outline">
                          <Settings className="h-3 w-3 mr-1" />
                          Configure
                        </Button>
                        <Button size="sm" variant="destructive">
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      )}

      {activeTab === "phases" && (
        <div className="space-y-4">
          <div className="flex justify-between">
            <div>
              <h2 className="text-xl font-semibold">Judging Phases</h2>
              <p className="text-sm text-muted-foreground">
                Manage judging rounds (Preliminary → Semifinal → Final)
              </p>
            </div>
            <Button onClick={() => setShowPhaseForm(!showPhaseForm)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Phase
            </Button>
          </div>

          {showPhaseForm && (
            <Card>
              <CardHeader>
                <CardTitle>New Judging Phase</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="phaseName">Phase Name</Label>
                  <Input id="phaseName" placeholder="e.g., Preliminary Round" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="phaseDescription">Description</Label>
                  <Textarea
                    id="phaseDescription"
                    placeholder="Describe this judging phase..."
                  />
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>Start Time</Label>
                    <Input type="datetime-local" />
                  </div>
                  <div className="space-y-2">
                    <Label>End Time</Label>
                    <Input type="datetime-local" />
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button>Create Phase</Button>
                  <Button
                    variant="outline"
                    onClick={() => setShowPhaseForm(false)}
                  >
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {phases.length === 0 ? (
            <Card>
              <CardContent className="py-12 text-center text-muted-foreground">
                No phases configured. Create your judging rounds.
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-2">
              {phases.map((phase, index) => (
                <Card key={phase.id}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold">
                          {index + 1}
                        </div>
                        <div>
                          <CardTitle>{phase.name}</CardTitle>
                          <CardDescription>
                            {phase.description || "No description"}
                          </CardDescription>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {phase.is_active ? (
                          <Button size="sm" variant="outline">
                            <Pause className="h-3 w-3 mr-1" />
                            Pause
                          </Button>
                        ) : (
                          <Button size="sm">
                            <Play className="h-3 w-3 mr-1" />
                            Start
                          </Button>
                        )}
                        <Button size="sm" variant="outline">
                          <Edit className="h-3 w-3 mr-1" />
                        </Button>
                        <Button size="sm" variant="outline">
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="text-sm text-muted-foreground">
                      {phase.start_time && (
                        <span>
                          Starts: {new Date(phase.start_time).toLocaleString()}
                        </span>
                      )}
                      {phase.end_time && (
                        <span className="ml-4">
                          Ends: {new Date(phase.end_time).toLocaleString()}
                        </span>
                      )}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
