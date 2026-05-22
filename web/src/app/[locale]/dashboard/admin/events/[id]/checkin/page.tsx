"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import { api } from "@/lib/api";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { QRCode } from "@/components/QRCode";
import { Scan, Users, CheckCircle, XCircle, RefreshCw } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

export default function CheckInPage() {
  const params = useParams();
  const eventId = params.id as string;
  const { toast } = useToast();
  
  const [loading, setLoading] = useState(true);
  const [qrCode, setQrCode] = useState<any>(null);
  const [stats, setStats] = useState<any>(null);
  const [attendees, setAttendees] = useState<any[]>([]);
  const [filter, setFilter] = useState<"all" | "checked_in" | "not_checked_in">("all");

  const loadData = useCallback(async () => {
    try {
      const [qrRes, statsRes, attendeesRes] = await Promise.all([
        api.get(`/api/core/checkin/events/${eventId}/qr`).catch(() => null),
        api.get(`/api/core/checkin/events/${eventId}/stats`).catch(() => null),
        api.get(`/api/core/checkin/events/${eventId}/attendees?checked_in=${filter === "all" ? "" : filter === "checked_in"}`).catch(() => null),
      ]);

      if (qrRes) setQrCode(qrRes);
      if (statsRes) setStats(statsRes);
      if (attendeesRes) setAttendees(attendeesRes.attendees || []);
    } catch (error) {
      console.error("Failed to load check-in data:", error);
      toast({ title: "Failed to load check-in data", variant: "destructive" });
    } finally {
      setLoading(false);
    }
  }, [eventId, filter, toast])

  useEffect(() => {
    loadData()
  }, [loadData])

  async function regenerateQR() {
    try {
      const res = await api.post(`/api/core/checkin/events/${eventId}/qr`);
      setQrCode(res);
      toast({ title: "QR code regenerated" });
    } catch (error) {
      toast({ title: "Failed to regenerate QR code", variant: "destructive" });
    }
  }

  if (loading) {
    return <div className="flex items-center justify-center h-screen">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Event Check-in</h1>
        <p className="text-muted-foreground">
          Manage QR code check-in for this event
        </p>
      </div>

      {/* QR Code Display */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Scan className="w-5 h-5" />
            Check-in QR Code
          </CardTitle>
          <CardDescription>
            Display this QR code for participants to scan at the event entrance
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col md:flex-row gap-6 items-center">
          {qrCode ? (
            <>
              <div className="bg-white p-4 rounded-lg border">
                <QRCode
                  value={qrCode.qr_url || qrCode.qr_image}
                  size={256}
                />
              </div>
              <div className="flex-1 space-y-4">
                <div>
                  <p className="text-sm font-medium mb-2">QR Code:</p>
                  <code className="block bg-muted p-2 rounded text-sm break-all">
                    {qrCode.code}
                  </code>
                </div>
                <Button
                  onClick={regenerateQR}
                  variant="outline"
                  size="sm"
                >
                  <RefreshCw className="w-4 h-4 mr-2" />
                  Regenerate QR Code
                </Button>
                <p className="text-sm text-muted-foreground">
                  Checked in: {qrCode.checked_in_count || 0} times
                </p>
              </div>
            </>
          ) : (
            <div className="text-center py-8">
              <p className="text-muted-foreground mb-4">No QR code generated yet</p>
              <Button onClick={regenerateQR}>
                Generate QR Code
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Statistics */}
      {stats && (
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total RSVPs</CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total_rsvps}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Checked In</CardTitle>
              <CheckCircle className="h-4 w-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{stats.checked_in}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Not Checked In</CardTitle>
              <XCircle className="h-4 w-4 text-red-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-red-600">{stats.not_checked_in}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Check-in Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.check_in_rate?.toFixed(1)}%</div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Attendees List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Attendees</CardTitle>
              <CardDescription>
                {attendees.length} {filter === "all" ? "total" : filter === "checked_in" ? "checked in" : "not checked in"}
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button
                variant={filter === "all" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("all")}
              >
                All
              </Button>
              <Button
                variant={filter === "checked_in" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("checked_in")}
              >
                Checked In
              </Button>
              <Button
                variant={filter === "not_checked_in" ? "default" : "outline"}
                size="sm"
                onClick={() => setFilter("not_checked_in")}
              >
                Not Checked In
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {attendees.length === 0 ? (
            <p className="text-muted-foreground text-center py-8">
              No attendees found
            </p>
          ) : (
            <div className="space-y-2">
              {attendees.map((attendee, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                      <Users className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <p className="font-medium">{attendee.user_name || "Unknown"}</p>
                      <p className="text-sm text-muted-foreground">{attendee.user_email}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    {attendee.checked_in ? (
                      <Badge variant="secondary" className="bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300">
                        <CheckCircle className="w-3 h-3 mr-1" />
                        Checked In
                      </Badge>
                    ) : (
                      <Badge variant="secondary" className="bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300">
                        <XCircle className="w-3 h-3 mr-1" />
                        Not Checked In
                      </Badge>
                    )}
                    {attendee.attended && (
                      <Badge variant="outline">Attended</Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
