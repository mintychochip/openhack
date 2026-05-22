"use client"

import { useState, useEffect, useCallback } from "react"
import { TrendingUp, Plus, Edit, Trash2, Calculator, Vote } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Formula {
  id: string
  name: string
  expression: string
  description?: string
  is_active: boolean
  created_at: string
}

interface VotingConfig {
  id: string
  is_enabled: boolean
  start_time?: string
  end_time?: string
  max_votes_per_user: number
  weight_participant: number
  weight_judge: number
  weight_admin: number
}

export default function AdminLeaderboardPage() {
  const { user } = useAuth()
  const [activeTab, setActiveTab] = useState<"formulas" | "voting" | "snapshots">("formulas")
  const [formulas, setFormulas] = useState<Formula[]>([])
  const [votingConfig, setVotingConfig] = useState<VotingConfig | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const { toast } = useToast()

  const loadData = useCallback(async () => {
    try {
      // These endpoints need to be added to API client
      // For now, using placeholder data
      setFormulas([
        {
          id: "1",
          name: "Default Formula",
          expression: "judge_score * 0.7 + public_votes * 0.3",
          description: "Weighted combination of judge scores and public votes",
          is_active: true,
          created_at: new Date().toISOString(),
        },
      ])
      setVotingConfig({
        id: "1",
        is_enabled: true,
        max_votes_per_user: 5,
        weight_participant: 1,
        weight_judge: 2,
        weight_admin: 3,
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load leaderboard config",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }, [toast])

  useEffect(() => {
    loadData()
  }, [loadData])

  if (!user || !user.roles.includes("admin")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Admin access required</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Leaderboard Settings</h1>
        <p className="text-muted-foreground">Configure ranking formulas and voting</p>
      </div>

      <div className="flex gap-2 border-b">
        <button
          onClick={() => setActiveTab("formulas")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "formulas"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          <Calculator className="inline h-4 w-4 mr-2" />
          Ranking Formulas
        </button>
        <button
          onClick={() => setActiveTab("voting")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "voting"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          <Vote className="inline h-4 w-4 mr-2" />
          Voting Config
        </button>
        <button
          onClick={() => setActiveTab("snapshots")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "snapshots"
              ? "border-primary text-foreground"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          <TrendingUp className="inline h-4 w-4 mr-2" />
          Snapshots
        </button>
      </div>

      {activeTab === "formulas" && (
        <div className="space-y-4">
          <div className="flex justify-between">
            <div>
              <h2 className="text-xl font-semibold">Ranking Formulas</h2>
              <p className="text-sm text-muted-foreground">
                Define how team scores are calculated
              </p>
            </div>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              New Formula
            </Button>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Formula Editor</CardTitle>
              <CardDescription>
                Use variables: judge_score, public_votes, team_size, bonus_points
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>Expression</Label>
                <Input placeholder="e.g., judge_score * 0.8 + public_votes * 0.2" />
              </div>
              <div className="space-y-2">
                <Label>Test Formula</Label>
                <div className="flex gap-2">
                  <Input placeholder="judge_score" className="w-32" />
                  <Input placeholder="public_votes" className="w-32" />
                  <Button variant="outline">Test</Button>
                </div>
              </div>
              <div className="p-4 bg-secondary rounded-lg">
                <p className="text-sm font-medium mb-2">Available Variables:</p>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• judge_score - Normalized judge score (0-100)</li>
                  <li>• public_votes - Total public votes received</li>
                  <li>• team_size - Number of team members</li>
                  <li>• bonus_points - Any bonus points awarded</li>
                </ul>
              </div>
            </CardContent>
          </Card>

          {formulas.map((formula) => (
            <Card key={formula.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle>{formula.name}</CardTitle>
                    <CardDescription className="font-mono text-xs mt-1">
                      {formula.expression}
                    </CardDescription>
                  </div>
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      formula.is_active
                        ? "bg-green-100 text-green-800"
                        : "bg-gray-100 text-gray-800"
                    }`}
                  >
                    {formula.is_active ? "Active" : "Inactive"}
                  </span>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between">
                  <p className="text-sm text-muted-foreground">
                    {formula.description}
                  </p>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline">
                      <Edit className="h-3 w-3 mr-1" />
                      Edit
                    </Button>
                    <Button size="sm" variant="outline">
                      <TrendingUp className="h-3 w-3 mr-1" />
                      Activate
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

      {activeTab === "voting" && votingConfig && (
        <div className="space-y-6">
          <div>
            <h2 className="text-xl font-semibold">Public Voting Configuration</h2>
            <p className="text-sm text-muted-foreground">
              Control how public voting works
            </p>
          </div>

          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Voting Settings</CardTitle>
                <div className="flex items-center gap-2">
                  <Label htmlFor="voting-enabled">Enable Voting</Label>
                  <Switch
                    id="voting-enabled"
                    checked={votingConfig.is_enabled}
                    onCheckedChange={(checked) =>
                      setVotingConfig({ ...votingConfig, is_enabled: checked })
                    }
                  />
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Voting Start Time</Label>
                  <Input type="datetime-local" />
                </div>
                <div className="space-y-2">
                  <Label>Voting End Time</Label>
                  <Input type="datetime-local" />
                </div>
              </div>
              <div className="space-y-2">
                <Label>Max Votes Per User</Label>
                <Input
                  type="number"
                  value={votingConfig.max_votes_per_user}
                  onChange={(e) =>
                    setVotingConfig({
                      ...votingConfig,
                      max_votes_per_user: parseInt(e.target.value),
                    })
                  }
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Vote Weights</CardTitle>
              <CardDescription>
                How much each role&apos;s votes count
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label>Participant Weight</Label>
                  <Input
                    type="number"
                    value={votingConfig.weight_participant}
                    onChange={(e) =>
                      setVotingConfig({
                        ...votingConfig,
                        weight_participant: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label>Judge Weight</Label>
                  <Input
                    type="number"
                    value={votingConfig.weight_judge}
                    onChange={(e) =>
                      setVotingConfig({
                        ...votingConfig,
                        weight_judge: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label>Admin Weight</Label>
                  <Input
                    type="number"
                    value={votingConfig.weight_admin}
                    onChange={(e) =>
                      setVotingConfig({
                        ...votingConfig,
                        weight_admin: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
              </div>
              <div className="p-4 bg-secondary rounded-lg">
                <p className="text-sm">
                  <strong>Example:</strong> If a project gets 1 vote from a participant (weight 1),
                  1 from a judge (weight 2), and 1 from an admin (weight 3), the total weighted votes = 6
                </p>
              </div>
            </CardContent>
          </Card>

          <div className="flex gap-2">
            <Button>Save Voting Config</Button>
            <Button variant="outline">Preview Leaderboard</Button>
          </div>
        </div>
      )}

      {activeTab === "snapshots" && (
        <Card>
          <CardHeader>
            <CardTitle>Leaderboard Snapshots</CardTitle>
            <CardDescription>
              Freeze leaderboard state at specific moments
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12 text-muted-foreground">
              <TrendingUp className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No snapshots yet</p>
              <p className="text-sm">
                Snapshots are created automatically when phases end
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
