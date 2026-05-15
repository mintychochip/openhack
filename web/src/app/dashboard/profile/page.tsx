"use client";

import { useState, useEffect } from "react";
import { useAuth } from "@/contexts/auth-context";
import { api } from "@/lib/api";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { User, Save, X, Plus } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

const EXPERIENCE_LEVELS = ["beginner", "intermediate", "advanced", "expert"];
const TSHIRT_SIZES = ["XS", "S", "M", "L", "XL", "XXL"];
const COMMON_SKILLS = [
  "Python", "JavaScript", "TypeScript", "React", "Node.js", "Go", "Rust",
  "Machine Learning", "Data Science", "Web Development", "Mobile Development",
  "DevOps", "Cloud", "Database", "API Design", "UI/UX", "Security"
];
const COMMON_INTERESTS = [
  "Fintech", "Health", "Sustainability", "Education", "AI/ML", "Blockchain",
  "IoT", "Gaming", "Social Good", "E-commerce", "Productivity", "Entertainment"
];

export default function ProfilePage() {
  const { user } = useAuth();
  const { toast } = useToast();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [profile, setProfile] = useState<any>({
    bio: "",
    skills: [],
    interests: [],
    experienceLevel: "",
    organization: "",
    timezone: "UTC",
    dietaryRestrictions: "",
    tshirtSize: "",
    phone: "",
    emergencyContact: { name: "", phone: "", relationship: "" },
  });

  const [newSkill, setNewSkill] = useState("");
  const [newInterest, setNewInterest] = useState("");

  useEffect(() => {
    loadProfile();
  }, []);

  async function loadProfile() {
    try {
      const data = await api.get("/api/auth/me/profile");
      setProfile({
        bio: data.bio || "",
        skills: data.skills || [],
        interests: data.interests || [],
        experienceLevel: data.experienceLevel || "",
        organization: data.organization || "",
        timezone: data.timezone || "UTC",
        dietaryRestrictions: data.dietaryRestrictions || "",
        tshirtSize: data.tshirtSize || "",
        phone: data.phone || "",
        emergencyContact: data.emergencyContact || { name: "", phone: "", relationship: "" },
      });
    } catch (error) {
      console.error("Failed to load profile:", error);
      toast({ title: "Failed to load profile", variant: "destructive" });
    } finally {
      setLoading(false);
    }
  }

  async function saveProfile() {
    setSaving(true);
    try {
      await api.put("/api/auth/me/profile", profile);
      toast({ title: "Profile saved!" });
    } catch (error) {
      console.error("Failed to save profile:", error);
      toast({ title: "Failed to save profile", variant: "destructive" });
    } finally {
      setSaving(false);
    }
  }

  function addSkill(skill: string) {
    if (skill && !profile.skills.includes(skill)) {
      setProfile({ ...profile, skills: [...profile.skills, skill] });
    }
    setNewSkill("");
  }

  function removeSkill(skill: string) {
    setProfile({ ...profile, skills: profile.skills.filter((s: string) => s !== skill) });
  }

  function addInterest(interest: string) {
    if (interest && !profile.interests.includes(interest)) {
      setProfile({ ...profile, interests: [...profile.interests, interest] });
    }
    setNewInterest("");
  }

  function removeInterest(interest: string) {
    setProfile({ ...profile, interests: profile.interests.filter((i: string) => i !== interest) });
  }

  if (loading) {
    return <div className="flex items-center justify-center h-screen">Loading profile...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Edit Profile</h1>
        <p className="text-muted-foreground">
          Update your skills, interests, and hackathon information
        </p>
      </div>

      {/* Basic Info */}
      <Card>
        <CardHeader>
          <CardTitle>Basic Information</CardTitle>
          <CardDescription>Your public profile information</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={user?.name || ""}
              disabled
              className="bg-muted"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              value={user?.email || ""}
              disabled
              className="bg-muted"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="bio">Bio</Label>
            <Textarea
              id="bio"
              value={profile.bio}
              onChange={(e) => setProfile({ ...profile, bio: e.target.value })}
              placeholder="Tell us about yourself..."
              rows={4}
            />
          </div>
        </CardContent>
      </Card>

      {/* Skills */}
      <Card>
        <CardHeader>
          <CardTitle>Skills & Expertise</CardTitle>
          <CardDescription>What technologies and skills do you bring to the team?</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-2">
            {profile.skills.map((skill: string) => (
              <Badge key={skill} variant="secondary" className="gap-1">
                {skill}
                <button
                  onClick={() => removeSkill(skill)}
                  className="ml-1 hover:text-red-500"
                >
                  <X className="w-3 h-3" />
                </button>
              </Badge>
            ))}
          </div>
          <div className="flex gap-2">
            <Input
              value={newSkill}
              onChange={(e) => setNewSkill(e.target.value)}
              placeholder="Add a skill"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addSkill(newSkill);
                }
              }}
            />
            <Button onClick={() => addSkill(newSkill)} size="icon">
              <Plus className="w-4 h-4" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2 mt-2">
            <span className="text-sm text-muted-foreground">Popular: </span>
            {COMMON_SKILLS.slice(0, 8).map((skill) => (
              <Badge
                key={skill}
                variant="outline"
                className="cursor-pointer hover:bg-secondary"
                onClick={() => addSkill(skill)}
              >
                {skill}
              </Badge>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Interests */}
      <Card>
        <CardHeader>
          <CardTitle>Interests</CardTitle>
          <CardDescription>What problem spaces excite you?</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-2">
            {profile.interests.map((interest: string) => (
              <Badge key={interest} variant="secondary" className="gap-1">
                {interest}
                <button
                  onClick={() => removeInterest(interest)}
                  className="ml-1 hover:text-red-500"
                >
                  <X className="w-3 h-3" />
                </button>
              </Badge>
            ))}
          </div>
          <div className="flex gap-2">
            <Input
              value={newInterest}
              onChange={(e) => setNewInterest(e.target.value)}
              placeholder="Add an interest"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addInterest(newInterest);
                }
              }}
            />
            <Button onClick={() => addInterest(newInterest)} size="icon">
              <Plus className="w-4 h-4" />
            </Button>
          </div>
          <div className="flex flex-wrap gap-2 mt-2">
            <span className="text-sm text-muted-foreground">Popular: </span>
            {COMMON_INTERESTS.slice(0, 8).map((interest) => (
              <Badge
                key={interest}
                variant="outline"
                className="cursor-pointer hover:bg-secondary"
                onClick={() => addInterest(interest)}
              >
                {interest}
              </Badge>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Hackathon Info */}
      <Card>
        <CardHeader>
          <CardTitle>Hackathon Information</CardTitle>
          <CardDescription>Details for event coordination</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-2">
          <div className="grid gap-2">
            <Label htmlFor="experienceLevel">Experience Level</Label>
            <Select
              value={profile.experienceLevel}
              onValueChange={(value) => setProfile({ ...profile, experienceLevel: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select level" />
              </SelectTrigger>
              <SelectContent>
                {EXPERIENCE_LEVELS.map((level) => (
                  <SelectItem key={level} value={level}>
                    {level.charAt(0).toUpperCase() + level.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="organization">Organization</Label>
            <Input
              id="organization"
              value={profile.organization}
              onChange={(e) => setProfile({ ...profile, organization: e.target.value })}
              placeholder="University or company"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="timezone">Timezone</Label>
            <Input
              id="timezone"
              value={profile.timezone}
              onChange={(e) => setProfile({ ...profile, timezone: e.target.value })}
              placeholder="e.g., America/New_York"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="tshirtSize">T-Shirt Size</Label>
            <Select
              value={profile.tshirtSize}
              onValueChange={(value) => setProfile({ ...profile, tshirtSize: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select size" />
              </SelectTrigger>
              <SelectContent>
                {TSHIRT_SIZES.map((size) => (
                  <SelectItem key={size} value={size}>
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="dietaryRestrictions">Dietary Restrictions</Label>
            <Textarea
              id="dietaryRestrictions"
              value={profile.dietaryRestrictions}
              onChange={(e) => setProfile({ ...profile, dietaryRestrictions: e.target.value })}
              placeholder="Vegetarian, gluten-free, etc."
              rows={2}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="phone">Phone Number</Label>
            <Input
              id="phone"
              value={profile.phone}
              onChange={(e) => setProfile({ ...profile, phone: e.target.value })}
              placeholder="+1 (555) 123-4567"
            />
          </div>
        </CardContent>
      </Card>

      {/* Emergency Contact */}
      <Card>
        <CardHeader>
          <CardTitle>Emergency Contact</CardTitle>
          <CardDescription>Who should we contact in case of emergency?</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-3">
          <div className="grid gap-2">
            <Label htmlFor="ec-name">Name</Label>
            <Input
              id="ec-name"
              value={profile.emergencyContact?.name || ""}
              onChange={(e) => setProfile({
                ...profile,
                emergencyContact: { ...profile.emergencyContact, name: e.target.value }
              })}
              placeholder="Contact name"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="ec-phone">Phone</Label>
            <Input
              id="ec-phone"
              value={profile.emergencyContact?.phone || ""}
              onChange={(e) => setProfile({
                ...profile,
                emergencyContact: { ...profile.emergencyContact, phone: e.target.value }
              })}
              placeholder="+1 (555) 123-4567"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="ec-relationship">Relationship</Label>
            <Input
              id="ec-relationship"
              value={profile.emergencyContact?.relationship || ""}
              onChange={(e) => setProfile({
                ...profile,
                emergencyContact: { ...profile.emergencyContact, relationship: e.target.value }
              })}
              placeholder="Friend, family, etc."
            />
          </div>
        </CardContent>
      </Card>

      {/* Save Button */}
      <div className="flex justify-end gap-4">
        <Button variant="outline" onClick={() => loadProfile()}>
          Cancel
        </Button>
        <Button onClick={saveProfile} disabled={saving}>
          <Save className="w-4 h-4 mr-2" />
          {saving ? "Saving..." : "Save Profile"}
        </Button>
      </div>
    </div>
  );
}
