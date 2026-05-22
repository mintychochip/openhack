"use client"

import { useState } from "react"
import { Briefcase, Plus, Edit, Trash2, ExternalLink, Eye } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Switch } from "@/components/ui/switch"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"

interface Booth {
  id: string
  sponsor_name: string
  title: string
  description?: string
  logo_url?: string
  website_url?: string
  is_published: boolean
  view_count: number
  created_at: string
}

export default function SponsorBoothPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [isLoading, setIsLoading] = useState(false)
  const [showForm, setShowForm] = useState(false)
  const [booths, setBooths] = useState<Booth[]>([
    {
      id: "1",
      sponsor_name: "Tech Corp",
      title: "Innovation Lab",
      description: "Building the future of technology",
      website_url: "https://techcorp.example.com",
      is_published: true,
      view_count: 234,
      created_at: new Date().toISOString(),
    },
  ])

  const [formData, setFormData] = useState({
    sponsor_name: "",
    title: "",
    description: "",
    website_url: "",
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
      // await api.sponsorCreateBooth(formData)
      toast({
        title: "Booth created",
        description: "Your sponsor booth has been created",
      })
      setShowForm(false)
      setFormData({ sponsor_name: "", title: "", description: "", website_url: "" })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to create booth",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleTogglePublish = async (boothId: string, currentStatus: boolean) => {
    try {
      // await api.sponsorUpdateBooth(boothId, { is_published: !currentStatus })
      toast({
        title: currentStatus ? "Booth unpublished" : "Booth published",
        description: `Your booth is now ${currentStatus ? "private" : "public"}`,
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to update booth",
        variant: "destructive",
      })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">Sponsor Booth</h1>
          <p className="text-muted-foreground">Manage your sponsor presence</p>
        </div>
        <Button onClick={() => setShowForm(!showForm)}>
          <Plus className="h-4 w-4 mr-2" />
          {showForm ? "Cancel" : "Create Booth"}
        </Button>
      </div>

      {showForm && (
        <Card>
          <CardHeader>
            <CardTitle>Create Sponsor Booth</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="sponsor_name">Sponsor Name</Label>
                <Input
                  id="sponsor_name"
                  value={formData.sponsor_name}
                  onChange={(e) =>
                    setFormData({ ...formData, sponsor_name: e.target.value })
                  }
                  placeholder="Your company name"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="title">Booth Title</Label>
                <Input
                  id="title"
                  value={formData.title}
                  onChange={(e) =>
                    setFormData({ ...formData, title: e.target.value })
                  }
                  placeholder="e.g., Innovation Lab"
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
                  placeholder="Describe your company and what you're looking for..."
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="website_url">Website URL</Label>
                <Input
                  id="website_url"
                  type="url"
                  value={formData.website_url}
                  onChange={(e) =>
                    setFormData({ ...formData, website_url: e.target.value })
                  }
                  placeholder="https://yourcompany.com"
                />
              </div>
              <div className="flex gap-2">
                <Button type="submit" disabled={isLoading}>
                  {isLoading ? "Creating..." : "Create Booth"}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setShowForm(false)
                    setFormData({ sponsor_name: "", title: "", description: "", website_url: "" })
                  }}
                >
                  Cancel
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      )}

      {booths.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            <Briefcase className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No booth yet. Create your sponsor booth to showcase your company.</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {booths.map((booth) => (
            <Card key={booth.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle>{booth.title}</CardTitle>
                    <CardDescription>{booth.sponsor_name}</CardDescription>
                  </div>
                  <div className="flex items-center gap-2">
                    <Label htmlFor={`publish-${booth.id}`}>Published</Label>
                    <Switch
                      id={`publish-${booth.id}`}
                      checked={booth.is_published}
                      onCheckedChange={() => handleTogglePublish(booth.id, booth.is_published)}
                    />
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-4">
                  {booth.description}
                </p>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4 text-sm text-muted-foreground">
                    <span>{booth.view_count} views</span>
                    {booth.website_url && (
                      <a
                        href={booth.website_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-1 text-primary hover:underline"
                      >
                        <ExternalLink className="h-3 w-3" />
                        Website
                      </a>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline">
                      <Eye className="h-3 w-3 mr-1" />
                      Preview
                    </Button>
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
