"use client"

import { useState } from "react"
import { Trophy, Plus, Edit, Trash2, Award } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Prize {
  id: string
  title: string
  description?: string
  value?: string
  winner_team_id?: string
  winner_team_name?: string
  status: "open" | "awarded"
  created_at: string
}

export default function SponsorPrizesPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [isLoading, setIsLoading] = useState(false)
  const [showForm, setShowForm] = useState(false)
  const [prizes, setPrizes] = useState<Prize[]>([
    {
      id: "1",
      title: "Best AI/ML Project",
      description: "For the most innovative use of artificial intelligence",
      value: "$1000",
      status: "open",
      created_at: new Date().toISOString(),
    },
  ])

  const [formData, setFormData] = useState({
    title: "",
    description: "",
    value: "",
  })

  if (!user || !user.roles.includes("sponsor")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Sponsor access required</p>
      </div>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)

    try {
      // await api.sponsorCreatePrize(formData)
      toast({
        title: "Prize created",
        description: "Your prize has been created",
      })
      setShowForm(false)
      setFormData({ title: "", description: "", value: "" })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to create prize",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleSelectWinner = async (prizeId: string) => {
    try {
      // Open modal to select winning team
      toast({
        title: "Select Winner",
        description: "Winner selection modal would open here",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to select winner",
        variant: "destructive",
      })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Prize Management</h1>
          <p className="text-muted-foreground">Create and manage your sponsor prizes</p>
        </div>
        <Button onClick={() => setShowForm(!showForm)}>
          <Plus className="h-4 w-4 mr-2" />
          {showForm ? "Cancel" : "Create Prize"}
        </Button>
      </div>

      {showForm && (
        <Card>
          <CardHeader>
            <CardTitle>Create Prize</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="title">Prize Title</Label>
                <Input
                  id="title"
                  value={formData.title}
                  onChange={(e) =>
                    setFormData({ ...formData, title: e.target.value })
                  }
                  placeholder="e.g., Best AI/ML Project"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={formData.description}
                  onChange={(e) =>
                    setFormData({ ...formData, description: e.target.value })
                  }
                  placeholder="Describe what makes a project eligible for this prize..."
                  rows={3}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="value">Prize Value</Label>
                <Input
                  id="value"
                  value={formData.value}
                  onChange={(e) =>
                    setFormData({ ...formData, value: e.target.value })
                  }
                  placeholder="e.g., $1000, Internship, Gift Card"
                />
              </div>
              <div className="flex gap-2">
                <Button type="submit" disabled={isLoading}>
                  {isLoading ? "Creating..." : "Create Prize"}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setShowForm(false)
                    setFormData({ title: "", description: "", value: "" })
                  }}
                >
                  Cancel
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      )}

      {prizes.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            <Trophy className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No prizes yet. Create your first sponsor prize.</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {prizes.map((prize) => (
            <Card key={prize.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <CardTitle>{prize.title}</CardTitle>
                      <span
                        className={`text-xs px-2 py-1 rounded ${
                          prize.status === "awarded"
                            ? "bg-green-100 text-green-800"
                            : "bg-blue-100 text-blue-800"
                        }`}
                      >
                        {prize.status}
                      </span>
                    </div>
                    <CardDescription>{prize.description}</CardDescription>
                  </div>
                  {prize.value && (
                    <div className="text-right">
                      <p className="text-lg font-bold text-primary">{prize.value}</p>
                    </div>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between">
                  <div>
                    {prize.winner_team_name ? (
                      <div className="flex items-center gap-2 text-green-600">
                        <Award className="h-4 w-4" />
                        <span className="font-medium">{prize.winner_team_name}</span>
                      </div>
                    ) : (
                      <p className="text-sm text-muted-foreground">No winner selected yet</p>
                    )}
                  </div>
                  <div className="flex gap-2">
                    {prize.status === "open" && (
                      <Button
                        size="sm"
                        onClick={() => handleSelectWinner(prize.id)}
                      >
                        <Award className="h-3 w-3 mr-1" />
                        Select Winner
                      </Button>
                    )}
                    <Button size="sm" variant="outline">
                      <Edit className="h-3 w-3 mr-1" />
                      Edit
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
  )
}
