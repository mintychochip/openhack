"use client"

import { useState, useEffect } from "react"
import { Calendar, MapPin, Users, Check } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { api } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { formatDateTime } from "@/lib/utils"

interface Event {
  id: string
  name: string
  description?: string
  location?: string
  start_time: string
  end_time?: string
  attendee_count: number
  max_attendees?: number
  is_rsvped?: boolean
}

export default function EventsPage() {
  const [events, setEvents] = useState<Event[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [rsvping, setRsvping] = useState<string | null>(null)
  const { toast } = useToast()

  useEffect(() => {
    loadEvents()
  }, [])

  const loadEvents = async () => {
    try {
      const data = await api.getEvents()
      setEvents(data.events || [])
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to load events",
        variant: "destructive",
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleRSVP = async (eventId: string) => {
    setRsvping(eventId)
    try {
      await api.rsvpEvent(eventId)
      toast({
        title: "RSVP confirmed",
        description: "See you at the event!",
      })
      loadEvents()
    } catch (error: any) {
      toast({
        title: "Error",
        description: error.message || "Failed to RSVP",
        variant: "destructive",
      })
    } finally {
      setRsvping(null)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Events</h1>
        <p className="text-muted-foreground">Hackathon schedule and activities</p>
      </div>

      {isLoading ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground">Loading events...</p>
        </div>
      ) : events.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <Calendar className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
            <p className="text-muted-foreground">No events scheduled yet</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {events.map((event) => (
            <Card key={event.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle>{event.name}</CardTitle>
                    <CardDescription className="line-clamp-2">
                      {event.description || "No description"}
                    </CardDescription>
                  </div>
                  {event.is_rsvped && (
                    <div className="flex items-center gap-1 text-green-600">
                      <Check className="h-4 w-4" />
                      <span className="text-sm">RSVP'd</span>
                    </div>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-4 mb-4">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Calendar className="h-4 w-4" />
                    <span>{formatDateTime(event.start_time)}</span>
                  </div>
                  {event.location && (
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <MapPin className="h-4 w-4" />
                      <span>{event.location}</span>
                    </div>
                  )}
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Users className="h-4 w-4" />
                    <span>
                      {event.attendee_count}
                      {event.max_attendees && ` / ${event.max_attendees}`}
                    </span>
                  </div>
                </div>
                {!event.is_rsvped && (
                  <Button
                    onClick={() => handleRSVP(event.id)}
                    disabled={rsvping === event.id}
                  >
                    {rsvping === event.id ? "RSVPing..." : "RSVP"}
                  </Button>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
