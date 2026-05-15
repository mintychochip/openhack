"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { Users } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Textarea } from "@/components/ui/textarea"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"

export default function CreateTeamPage() {
  const router = useRouter()
  const { toast } = useToast()
  const [isLoading, setIsLoading] = useState(false)
  const [formData, setFormData] = useState({
    name: "",
    description: "",
    maxSize: "4",
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)

    try {
      const result = await api.createTeam({
        name: formData.name,
        description: formData.description || undefined,
        maxSize: parseInt(formData.maxSize),
      })
      toast({
        title: "Team created",
        description: "Your team has been created successfully",
      })
      router.push(`/dashboard/teams/${result.id}`)
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to create team",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Create Team</h1>
        <p className="text-muted-foreground">Start your hackathon team</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            <CardTitle>Team Details</CardTitle>
          </div>
          <CardDescription>
            Fill in the information for your new team
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Team Name</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                placeholder="Awesome Hackers"
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
                placeholder="What's your team about? What are you building?"
                rows={4}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="maxSize">Maximum Team Size</Label>
              <Input
                id="maxSize"
                type="number"
                min="2"
                max="10"
                value={formData.maxSize}
                onChange={(e) =>
                  setFormData({ ...formData, maxSize: e.target.value })
                }
              />
              <p className="text-sm text-muted-foreground">
                Maximum number of team members (2-10)
              </p>
            </div>
            <div className="flex gap-2">
              <Button type="submit" disabled={isLoading}>
                {isLoading ? "Creating..." : "Create Team"}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => router.back()}
              >
                Cancel
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
