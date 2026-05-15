"use client"

import { useState, useEffect } from "react"
import { Trophy, TrendingUp, Users, Medal } from "lucide-react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"

interface Ranking {
  rank: number
  team_id: string
  team_name: string
  combined_score: number
  votes?: number
}

export default function LeaderboardPage() {
  const [rankings, setRankings] = useState<Ranking[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isVoting, setIsVoting] = useState<string | null>(null)
  const { toast } = useToast()

  useEffect(() => {
    loadLeaderboard()
  }, [])

  const loadLeaderboard = async () => {
    try {
      const data = await api.getLeaderboard(50)
      setRankings(data.rankings || [])
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load leaderboard",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleVote = async (teamId: string) => {
    setIsVoting(teamId)
    try {
      await api.castVote(teamId)
      toast({
        title: "Vote recorded",
        description: "Thanks for voting!",
      })
      loadLeaderboard()
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to cast vote",
        variant: "destructive",
      })
    } finally {
      setIsVoting(null)
    }
  }

  const getRankIcon = (rank: number) => {
    if (rank === 1) return <Medal className="h-5 w-5 text-yellow-500" />
    if (rank === 2) return <Medal className="h-5 w-5 text-gray-400" />
    if (rank === 3) return <Medal className="h-5 w-5 text-amber-600" />
    return <span className="text-lg font-bold w-5 text-center">{rank}</span>
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Leaderboard</h1>
        <p className="text-muted-foreground">Real-time rankings</p>
      </div>

      <div className="grid md:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Teams</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{rankings.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Top Score</CardTitle>
            <Trophy className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {rankings[0]?.combined_score.toFixed(1) || "-"}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Your Rank</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">-</div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Rankings</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading rankings...</p>
            </div>
          ) : rankings.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              No rankings yet
            </div>
          ) : (
            <div className="space-y-2">
              {rankings.map((ranking) => (
                <div
                  key={ranking.team_id}
                  className="flex items-center justify-between p-4 border rounded-lg hover:bg-secondary/50 transition-colors"
                >
                  <div className="flex items-center gap-4">
                    <div className="w-8 flex justify-center">
                      {getRankIcon(ranking.rank)}
                    </div>
                    <div>
                      <p className="font-medium">{ranking.team_name}</p>
                      {ranking.votes !== undefined && (
                        <p className="text-sm text-muted-foreground">
                          {ranking.votes} votes
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-4">
                    <div className="text-right">
                      <p className="text-2xl font-bold">
                        {ranking.combined_score.toFixed(1)}
                      </p>
                      <p className="text-xs text-muted-foreground">points</p>
                    </div>
                    <Button
                      size="sm"
                      onClick={() => handleVote(ranking.team_id)}
                      disabled={isVoting === ranking.team_id}
                    >
                      {isVoting === ranking.team_id ? "Voting..." : "Vote"}
                    </Button>
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
