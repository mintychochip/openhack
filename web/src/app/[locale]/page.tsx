"use client"

import { useState, useEffect } from "react"
import Link from "next/link"
import { ArrowRight, Code, Users, Trophy, Zap } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useTheme } from "@/contexts/theme-context"

export default function LandingPage() {
  const { config } = useTheme()
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
  }, [])

  const name = mounted ? (config?.name || "OpenHack") : "OpenHack"
  const tagline = mounted ? (config?.tagline || "Build something amazing") : "Build something amazing"

  return (
    <div className="min-h-screen bg-gradient-to-b from-background to-secondary font-body">
      <header className="container mx-auto px-4 py-6">
        <nav className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Code className="h-8 w-8 text-primary" />
            <span className="text-2xl font-bold font-display">{name}</span>
          </div>
          <div className="flex items-center gap-4">
            <Link href="/login">
              <Button variant="ghost">Login</Button>
            </Link>
            <Link href="/register">
              <Button>Get Started</Button>
            </Link>
          </div>
        </nav>
      </header>

      <main className="container mx-auto px-4 py-16">
        <section className="text-center mb-20">
          <h1 className="text-5xl font-bold mb-6 font-display">
            {name}
            <span className="text-primary"> Hackathon</span>
          </h1>
          <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
            {tagline}. Self-hosted, modular, and AI-powered. Everything you need to run an amazing hackathon
            for universities, companies, communities, and conferences.
          </p>
          <div className="flex gap-4 justify-center">
            <Link href="/register">
              <Button size="lg" className="gap-2">
                Start Free <ArrowRight className="h-4 w-4" />
              </Button>
            </Link>
            <Link href="#features">
              <Button size="lg" variant="outline">
                Learn More
              </Button>
            </Link>
          </div>
        </section>

        <section id="features" className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 mb-20">
          <Card>
            <CardHeader>
              <Users className="h-10 w-10 text-primary mb-2" />
              <CardTitle className="font-heading">Team Management</CardTitle>
              <CardDescription>
                Create teams, invite members, manage rosters
              </CardDescription>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader>
              <Code className="h-10 w-10 text-primary mb-2" />
              <CardTitle className="font-heading">Project Submissions</CardTitle>
              <CardDescription>
                Submit projects with demos, code repos, and presentations
              </CardDescription>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader>
              <Trophy className="h-10 w-10 text-primary mb-2" />
              <CardTitle className="font-heading">Fair Judging</CardTitle>
              <CardDescription>
                Multi-phase judging with rubrics and score normalization
              </CardDescription>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader>
              <Zap className="h-10 w-10 text-primary mb-2" />
              <CardTitle className="font-heading">Real-time Leaderboards</CardTitle>
              <CardDescription>
                Live rankings with configurable formulas and public voting
              </CardDescription>
            </CardHeader>
          </Card>
        </section>

        <section className="text-center mb-20">
          <h2 className="text-3xl font-bold mb-4 font-heading">Why {name}?</h2>
          <p className="text-muted-foreground max-w-2xl mx-auto mb-8">
            Built for organizers who want full control over their hackathon platform.
            Self-hosted, customizable, and ready to scale.
          </p>
          <div className="grid md:grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="font-heading">Self-Hosted</CardTitle>
                <CardDescription>
                  Deploy on your own infrastructure with Docker or Kubernetes
                </CardDescription>
              </CardHeader>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="font-heading">Modular Design</CardTitle>
                <CardDescription>
                  Swap auth providers, mail services, and storage backends
                </CardDescription>
              </CardHeader>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="font-heading">AI-Native</CardTitle>
                <CardDescription>
                  Built-in AI assistant for idea generation and team matching
                </CardDescription>
              </CardHeader>
            </Card>
          </div>
        </section>

        <section className="text-center">
          <h2 className="text-3xl font-bold mb-4 font-heading">Ready to Start?</h2>
          <p className="text-muted-foreground mb-8">
            Create your account and launch your hackathon in minutes.
          </p>
          <Link href="/register">
            <Button size="lg" className="gap-2">
              Create Account <ArrowRight className="h-4 w-4" />
            </Button>
          </Link>
        </section>
      </main>

      <footer className="border-t mt-20 py-8">
        <div className="container mx-auto px-4 text-center text-muted-foreground">
          <p>&copy; 2026 {name}. Self-hosted hackathon platform.</p>
        </div>
      </footer>
    </div>
  )
}
