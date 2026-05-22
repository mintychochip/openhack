"use client"

import { useState } from "react"
import { FileCheck, Check, X, Eye } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { useToast } from "@/hooks/use-toast"
import { useAuth } from "@/contexts/auth-context"
import Link from "next/link"

interface Submission {
  id: string
  project_id: string
  project_title: string
  team_name: string
  category: string
  submitted_at: string
  status: "pending" | "approved" | "rejected"
}

export default function SponsorSubmissionsPage() {
  const { user } = useAuth()
  const { toast } = useToast()
  const [submissions, setSubmissions] = useState<Submission[]>([
    {
      id: "1",
      project_id: "proj-1",
      project_title: "AI-Powered Study Buddy",
      team_name: "Code Wizards",
      category: "Best AI/ML Project",
      submitted_at: new Date().toISOString(),
      status: "pending",
    },
    {
      id: "2",
      project_id: "proj-2",
      project_title: "Green Energy Tracker",
      team_name: "Eco Hackers",
      category: "Best AI/ML Project",
      submitted_at: new Date().toISOString(),
      status: "approved",
    },
  ])

  if (!user || !user.roles.includes("sponsor")) {
    return (
      <div className="text-center py-12">
        <h1 className="text-2xl font-bold mb-2">Access Denied</h1>
        <p className="text-muted-foreground">Sponsor access required</p>
      </div>
    )
  }

  const handleApprove = async (submissionId: string) => {
    try {
      toast({
        title: "Submission approved",
        description: "The project has been approved for your prize",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to approve submission",
        variant: "destructive",
      })
    }
  }

  const handleReject = async (submissionId: string) => {
    try {
      toast({
        title: "Submission rejected",
        description: "The project has been rejected",
      })
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to reject submission",
        variant: "destructive",
      })
    }
  }

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "approved":
        return <Badge className="bg-green-500">Approved</Badge>
      case "rejected":
        return <Badge variant="destructive">Rejected</Badge>
      default:
        return <Badge variant="secondary">Pending</Badge>
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Prize Submissions</h1>
        <p className="text-muted-foreground">Review projects submitted for your prizes</p>
      </div>

      {submissions.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            <FileCheck className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No submissions yet for your prizes</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {submissions.map((submission) => (
            <Card key={submission.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-2">
                      <CardTitle>{submission.project_title}</CardTitle>
                      {getStatusBadge(submission.status)}
                    </div>
                    <CardDescription>
                      by {submission.team_name} • {submission.category}
                    </CardDescription>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Submitted {new Date(submission.submitted_at).toLocaleDateString()}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between">
                  <Link href={`/dashboard/projects/${submission.project_id}`}>
                    <Button size="sm" variant="outline">
                      <Eye className="h-3 w-3 mr-1" />
                      View Project
                    </Button>
                  </Link>
                  {submission.status === "pending" && (
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        onClick={() => handleApprove(submission.id)}
                        className="bg-green-600 hover:bg-green-700"
                      >
                        <Check className="h-3 w-3 mr-1" />
                        Approve
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleReject(submission.id)}
                      >
                        <X className="h-3 w-3 mr-1" />
                        Reject
                      </Button>
                    </div>
                  )}
                  {submission.status === "approved" && (
                    <p className="text-sm text-green-600">
                      ✓ Approved for prize consideration
                    </p>
                  )}
                  {submission.status === "rejected" && (
                    <p className="text-sm text-destructive">
                      ✗ Not eligible for prize
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
