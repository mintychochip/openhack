/**
 * API client for OpenHack backend services.
 *
 * Expected Behavior:
 *   Provides typed methods for all API endpoints.
 *   Handles authentication via JWT tokens stored in localStorage.
 *   Automatically refreshes access tokens on 401 responses using the
 *   refresh token, retrying the original request once after a successful
 *   refresh. If the refresh fails, both tokens are cleared and the user
 *   is redirected to /login. Concurrent 401s are coalesced into a single
 *   refresh call.
 *   Throws ApiError on non-2xx responses.
 *
 * Side Effects:
 *   Reads from and writes to localStorage keys "access_token" and
 *   "refresh_token" on init, login, refresh, and clearAuth.
 *   Redirects to /login via window.location.assign on auth failure.
 */

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || ""

function buildUrl(endpoint: string): string {
  if (API_BASE_URL) {
    return `${API_BASE_URL}${endpoint}`
  }
  return endpoint
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message)
    this.name = "ApiError"
  }
}

interface RequestOptions {
  method?: string
  body?: any
  headers?: Record<string, string>
  requiresAuth?: boolean
}

class ApiClient {
  private token: string | null = null
  private refreshToken: string | null = null
  private refreshPromise: Promise<string | null> | null = null

  constructor() {
    if (typeof window !== "undefined") {
      this.token = localStorage.getItem("access_token")
      this.refreshToken = localStorage.getItem("refresh_token")
    }
  }

  setToken(token: string | null) {
    this.token = token
    if (typeof window !== "undefined") {
      if (token) {
        localStorage.setItem("access_token", token)
        document.cookie = `access_token=${token}; path=/; max-age=86400; SameSite=Lax`
      } else {
        localStorage.removeItem("access_token")
        document.cookie = "access_token=; path=/; max-age=0"
      }
    }
  }

  private setRefreshToken(refreshToken: string | null) {
    this.refreshToken = refreshToken
    if (typeof window !== "undefined") {
      if (refreshToken) {
        localStorage.setItem("refresh_token", refreshToken)
      } else {
        localStorage.removeItem("refresh_token")
      }
    }
  }

  getToken(): string | null {
    return this.token
  }

  /**
   * Whether the client currently holds an access token.
   *
   * @expectedBehavior Returns true when an access_token is present in
   * memory (populated from localStorage on construction or set via
   * login/refresh). Returns false otherwise. Does NOT validate whether
   * the token is expired — only checks presence.
   */
  get isAuthenticated(): boolean {
    return !!this.token
  }

  /**
   * Clear both access and refresh tokens from memory and localStorage.
   *
   * @expectedBehavior Unconditionally removes "access_token" and
   * "refresh_token" from localStorage, nulls the in-memory token
   * fields. Does NOT redirect — callers handle navigation.
   *
   * @sideeffect Mutates localStorage (removes two keys).
   */
  clearAuth() {
    this.setToken(null)
    this.setRefreshToken(null)
  }

  logout() {
    this.setToken(null)
    this.setRefreshToken(null)
  }

  /**
   * Refresh the access token using the stored refresh token.
   *
   * @expectedBehavior Reads the refresh_token from in-memory state
   * (originally loaded from localStorage). Calls
   * POST /api/auth/refresh with { refresh_token }. On success,
   * stores the new access_token via setToken and returns it.
   * Coalesces concurrent calls — if a refresh is already in flight,
   * returns the same promise instead of starting a second request.
   * On failure (network error, non-2xx, or missing refresh token),
   * calls clearAuth() and returns null.
   *
   * @returns The new access_token string on success, or null on
   * failure (after clearing auth state and redirecting).
   *
   * @sideeffect Writes new access_token to localStorage on success.
   * @sideeffect Calls clearAuth on failure, which removes both tokens
   * from localStorage and redirects to /login.
   */
  private async refresh(): Promise<string | null> {
    if (this.refreshPromise) {
      return this.refreshPromise
    }

    this.refreshPromise = this._doRefresh()
    try {
      return await this.refreshPromise
    } finally {
      this.refreshPromise = null
    }
  }

  private async _doRefresh(): Promise<string | null> {
    if (!this.refreshToken) {
      this.clearAuth()
      return null
    }

    try {
      const response = await fetch(buildUrl("/api/auth/refresh"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: this.refreshToken }),
      })

      if (!response.ok) {
        this.clearAuth()
        return null
      }

      const data = await response.json()
      this.setToken(data.access_token)
      return data.access_token
    } catch {
      this.clearAuth()
      return null
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestOptions = {},
  ): Promise<T> {
    const {
      method = "GET",
      body,
      headers = {},
      requiresAuth = true,
    } = options

    const url = buildUrl(endpoint)

    const config: RequestInit = {
      method,
      headers: {
        "Content-Type": "application/json",
        ...headers,
      },
    }

    if (requiresAuth && this.token) {
      ;(config.headers as Record<string, string>)["Authorization"] = `Bearer ${this.token}`
    }

    if (body) {
      config.body = JSON.stringify(body)
    }

    const response = await fetch(url, config)

    if (response.status === 401 && requiresAuth) {
      const newToken = await this.refresh()
      if (newToken) {
        ;(config.headers as Record<string, string>)["Authorization"] = `Bearer ${newToken}`
        const retryResponse = await fetch(url, config)
        if (retryResponse.status === 401) {
          this.clearAuth()
          throw new ApiError(401, "SESSION_EXPIRED", "Session expired")
        }
        if (!retryResponse.ok) {
          const errorData = await retryResponse.json().catch(() => ({}))
          throw new ApiError(
            retryResponse.status,
            errorData.code || "UNKNOWN_ERROR",
            errorData.message || retryResponse.statusText,
          )
        }
        return retryResponse.json()
      }
      throw new ApiError(401, "SESSION_EXPIRED", "Session expired")
    }

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new ApiError(
        response.status,
        errorData.code || "UNKNOWN_ERROR",
        errorData.message || response.statusText,
      )
    }

    return response.json()
  }

  async get<T = any>(endpoint: string) {
    return this.request<T>(endpoint)
  }

  async post<T = any>(endpoint: string, body?: any) {
    return this.request<T>(endpoint, {
      method: "POST",
      body,
    })
  }

  async put<T = any>(endpoint: string, body?: any) {
    return this.request<T>(endpoint, {
      method: "PUT",
      body,
    })
  }

  async delete<T = any>(endpoint: string) {
    return this.request<T>(endpoint, {
      method: "DELETE",
    })
  }

  // ==================== Auth Endpoints ====================

  /**
   * Log in with email and password.
   *
   * @expectedBehavior Sends credentials to POST /api/auth/login.
   * On success, stores both access_token and refresh_token in
   * localStorage and in-memory state, then returns the full
   * response including the user object.
   *
   * @sideeffect Writes "access_token" and "refresh_token" to
   * localStorage on success.
   */
  async login(email: string, password: string) {
    const data = await this.request<{
      accessToken: string
      refreshToken: string
      user: any
    }>("/api/auth/login", {
      method: "POST",
      body: { email, password },
      requiresAuth: false,
    })

    this.setToken(data.accessToken)
    this.setRefreshToken(data.refreshToken)
    return data
  }

  async register(email: string, password: string, name: string) {
    return this.request<{ user_id: string }>("/api/auth/register", {
      method: "POST",
      body: { email, password, name },
      requiresAuth: false,
    })
  }

  async forgotPassword(email: string) {
    return this.request<{ message: string }>("/api/auth/forgot-password", {
      method: "POST",
      body: { email },
      requiresAuth: false,
    })
  }

  async resetPassword(token: string, newPassword: string) {
    return this.request<{ message: string }>("/api/auth/reset-password", {
      method: "POST",
      body: { token, newPassword },
      requiresAuth: false,
    })
  }

  async getMe() {
    return this.request<any>("/api/auth/me")
  }

  async updateMe(data: { name?: string; githubUsername?: string }) {
    return this.request<any>("/api/auth/me", {
      method: "PUT",
      body: data,
    })
  }

  async getProfile() {
    return this.request<any>("/api/auth/me/profile")
  }

  async updateProfile(data: {
    bio?: string;
    skills?: string[];
    interests?: string[];
    experienceLevel?: string;
    organization?: string;
    timezone?: string;
    dietaryRestrictions?: string;
    tshirtSize?: string;
    phone?: string;
    emergencyContact?: { name: string; phone: string; relationship: string };
  }) {
    return this.request<any>("/api/auth/me/profile", {
      method: "PUT",
      body: data,
    })
  }

  async getUserProfile(userId: string) {
    return this.request<any>(`/api/auth/users/${userId}/profile`)
  }

  async searchUsersBySkills(skills: string, interests?: string, limit?: number) {
    const params = new URLSearchParams({ skills });
    if (interests) params.append('interests', interests);
    if (limit) params.append('limit', limit.toString());
    return this.request<any>(`/api/auth/users/search?${params}`);
  }

  async uploadAvatar(file: File) {
    const formData = new FormData()
    formData.append("file", file)

    const headers: Record<string, string> = {}
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`
    }

    const response = await fetch(buildUrl("/api/auth/me/avatar"), {
      method: "POST",
      headers,
      body: formData,
    })

    if (response.status === 401) {
      const newToken = await this.refresh()
      if (newToken) {
        const retryResponse = await fetch(buildUrl("/api/auth/me/avatar"), {
          method: "POST",
          headers: { Authorization: `Bearer ${newToken}` },
          body: formData,
        })
        if (!retryResponse.ok) {
          const errorData = await retryResponse.json().catch(() => ({}))
          throw new ApiError(
            retryResponse.status,
            errorData.code || "UNKNOWN_ERROR",
            errorData.message || retryResponse.statusText,
          )
        }
        return retryResponse.json()
      }
      this.clearAuth()
      throw new ApiError(401, "SESSION_EXPIRED", "Session expired")
    }

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new ApiError(
        response.status,
        errorData.code || "UNKNOWN_ERROR",
        errorData.message || response.statusText,
      )
    }

    return response.json()
  }

  async deleteAvatar() {
    return this.request<{ deleted: boolean }>("/api/auth/me/avatar", {
      method: "DELETE",
    })
  }

  // ==================== Core Endpoints ====================

  async getTeams() {
    return this.request<{ teams: any[] }>("/api/core/teams")
  }

  async createTeam(data: { name: string; description?: string; maxSize?: number }) {
    return this.request<{ id: string }>("/api/core/teams", {
      method: "POST",
      body: data,
    })
  }

  async getTeam(id: string) {
    return this.request<any>(`/api/core/teams/${id}`)
  }

  async joinTeam(id: string, joinCode: string) {
    return this.request<{ joined: true }>(`/api/core/teams/${id}/join`, {
      method: "POST",
      body: { joinCode },
    })
  }

  async getProjects(params?: { page?: number; limit?: number; category?: string }) {
    const queryString = new URLSearchParams(params as any).toString()
    return this.request<{ projects: any[]; total: number }>(
      `/api/core/projects${queryString ? `?${queryString}` : ""}`,
    )
  }

  async createProject(data: {
    team_id: string
    title: string
    description?: string
    category?: string
    repo_url?: string
    demo_url?: string
    tags?: string[]
  }) {
    return this.request<{ project_id: string }>("/api/core/projects", {
      method: "POST",
      body: data,
    })
  }

  async getProject(id: string) {
    return this.request<any>(`/api/core/projects/${id}`)
  }

  async updateProject(id: string, data: any) {
    return this.request<{ updated: true }>(`/api/core/projects/${id}`, {
      method: "PUT",
      body: data,
    })
  }

  async submitProject(id: string) {
    return this.request<any>(`/api/core/projects/${id}/submit`, {
      method: "POST",
    })
  }

  async uploadProjectDemo(id: string, file: File) {
    const formData = new FormData()
    formData.append("file", file)

    const headers: Record<string, string> = {}
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`
    }

    const response = await fetch(buildUrl(`/api/core/projects/${id}/demo`), {
      method: "POST",
      headers,
      body: formData,
    })

    if (response.status === 401) {
      const newToken = await this.refresh()
      if (newToken) {
        const retryResponse = await fetch(buildUrl(`/api/core/projects/${id}/demo`), {
          method: "POST",
          headers: { Authorization: `Bearer ${newToken}` },
          body: formData,
        })
        if (!retryResponse.ok) {
          const errorData = await retryResponse.json().catch(() => ({}))
          throw new ApiError(
            retryResponse.status,
            errorData.code || "UNKNOWN_ERROR",
            errorData.message || retryResponse.statusText,
          )
        }
        return retryResponse.json()
      }
      this.clearAuth()
      throw new ApiError(401, "SESSION_EXPIRED", "Session expired")
    }

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new ApiError(
        response.status,
        errorData.code || "UNKNOWN_ERROR",
        errorData.message || response.statusText,
      )
    }

    return response.json()
  }

  async getEvents() {
    return this.request<{ events: any[] }>("/api/core/events")
  }

  async rsvpEvent(id: string) {
    return this.request<{ rsvped: true }>(`/api/core/events/${id}/rsvp`, {
      method: "POST",
    })
  }

  // ==================== Judging Endpoints ====================

  async getRubrics() {
    return this.request<{ rubrics: any[] }>("/api/judging/rubrics")
  }

  async getMyAssignments() {
    return this.request<{ assignments: any[] }>("/api/judging/assignments/me")
  }

  async submitScore(assignment_id: string, scores: Record<string, number>, comment?: string) {
    return this.request<{ score_id: string }>("/api/judging/scores", {
      method: "POST",
      body: { assignment_id, scores, comment },
    })
  }

  // ==================== Leaderboard Endpoints ====================

  async getLeaderboard(limit = 50) {
    return this.request<{
      rankings: any[]
      total_teams: number
      last_updated: string
    }>(`/api/leaderboard?limit=${limit}`)
  }

  async getTeamRank(team_id: string) {
    return this.request<any>(`/api/leaderboard/team/${team_id}`)
  }

  async castVote(project_id: string) {
    return this.request<{ vote_recorded: true }>("/api/leaderboard/votes", {
      method: "POST",
      body: { project_id },
    })
  }

  // ==================== Media Endpoints ====================

  async uploadFile(file: File, folder = "uploads") {
    const formData = new FormData()
    formData.append("file", file)
    formData.append("folder", folder)

    const headers: Record<string, string> = {}
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`
    }

    const response = await fetch(buildUrl("/api/media/upload"), {
      method: "POST",
      headers,
      body: formData,
    })

    if (response.status === 401) {
      const newToken = await this.refresh()
      if (newToken) {
        const retryResponse = await fetch(buildUrl("/api/media/upload"), {
          method: "POST",
          headers: { Authorization: `Bearer ${newToken}` },
          body: formData,
        })
        if (!retryResponse.ok) {
          const errorData = await retryResponse.json().catch(() => ({}))
          throw new ApiError(
            retryResponse.status,
            errorData.code || "UNKNOWN_ERROR",
            errorData.message || retryResponse.statusText,
          )
        }
        return retryResponse.json()
      }
      this.clearAuth()
      throw new ApiError(401, "SESSION_EXPIRED", "Session expired")
    }

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new ApiError(
        response.status,
        errorData.code || "UNKNOWN_ERROR",
        errorData.message || response.statusText,
      )
    }

    return response.json()
  }

  async deleteFile(file_id: string) {
    return this.request<{ deleted: boolean }>(`/api/media/${file_id}`, {
      method: "DELETE",
    })
  }

  // ==================== Admin Endpoints ====================

  async adminGetUsers() {
    return this.request<{ users: any[] }>("/api/admin/users")
  }

  async adminDeleteUser(userId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/users/${userId}`, {
      method: "DELETE",
    })
  }

  async adminUpdateUserRole(userId: string, role: string) {
    return this.request<{ updated: true }>(`/api/admin/users/${userId}/role`, {
      method: "PUT",
      body: { role },
    })
  }

  async adminGetTeams() {
    return this.request<{ teams: any[] }>("/api/admin/teams")
  }

  async adminDeleteTeam(teamId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/teams/${teamId}`, {
      method: "DELETE",
    })
  }

  async adminGetProjects() {
    return this.request<{ projects: any[] }>("/api/admin/projects")
  }

  async adminDeleteProject(projectId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/projects/${projectId}`, {
      method: "DELETE",
    })
  }

  async adminUpdateProjectStatus(projectId: string, status: string) {
    return this.request<{ updated: true }>(`/api/admin/projects/${projectId}/status`, {
      method: "PUT",
      body: { status },
    })
  }

  async adminGetRubrics() {
    return this.request<{ rubrics: any[] }>("/api/admin/judging/rubrics")
  }

  async adminCreateRubric(data: any) {
    return this.request<{ id: string }>("/api/admin/judging/rubrics", {
      method: "POST",
      body: data,
    })
  }

  async adminGetPhases() {
    return this.request<{ phases: any[] }>("/api/admin/judging/phases")
  }

  async adminCreatePhase(data: any) {
    return this.request<{ id: string }>("/api/admin/judging/phases", {
      method: "POST",
      body: data,
    })
  }

  async adminUpdatePhase(phaseId: string, data: any) {
    return this.request<{ updated: true }>(`/api/admin/judging/phases/${phaseId}`, {
      method: "PUT",
      body: data,
    })
  }

  async adminDeletePhase(phaseId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/judging/phases/${phaseId}`, {
      method: "DELETE",
    })
  }

  async adminGetFormulas() {
    return this.request<{ formulas: any[] }>("/api/admin/leaderboard/formulas")
  }

  async adminCreateFormula(data: any) {
    return this.request<{ id: string }>("/api/admin/leaderboard/formulas", {
      method: "POST",
      body: data,
    })
  }

  async adminActivateFormula(formulaId: string) {
    return this.request<{ activated: true }>(`/api/admin/leaderboard/formulas/${formulaId}/activate`, {
      method: "POST",
    })
  }

  async adminGetVotingConfig() {
    return this.request<any>("/api/admin/leaderboard/voting")
  }

  async adminUpdateVotingConfig(data: any) {
    return this.request<{ updated: true }>("/api/admin/leaderboard/voting", {
      method: "PUT",
      body: data,
    })
  }

  async adminGetEvents() {
    return this.request<{ events: any[] }>("/api/admin/events")
  }

  async adminCreateEvent(data: any) {
    return this.request<{ id: string }>("/api/admin/events", {
      method: "POST",
      body: data,
    })
  }

  async adminUpdateEvent(eventId: string, data: any) {
    return this.request<{ updated: true }>(`/api/admin/events/${eventId}`, {
      method: "PUT",
      body: data,
    })
  }

  async adminDeleteEvent(eventId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/events/${eventId}`, {
      method: "DELETE",
    })
  }

  async adminGetHackathonConfig() {
    return this.request<any>("/api/admin/hackathon")
  }

  async adminUpdateHackathonConfig(data: any) {
    return this.request<{ updated: true }>("/api/admin/hackathon", {
      method: "PUT",
      body: data,
    })
  }

  async adminGetMailTemplates() {
    return this.request<{ templates: any[] }>("/api/admin/mail/templates")
  }

  async adminCreateMailTemplate(data: any) {
    return this.request<{ id: string }>("/api/admin/mail/templates", {
      method: "POST",
      body: data,
    })
  }

  async adminUpdateMailTemplate(templateId: string, data: any) {
    return this.request<{ updated: true }>(`/api/admin/mail/templates/${templateId}`, {
      method: "PUT",
      body: data,
    })
  }

  async adminDeleteMailTemplate(templateId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/mail/templates/${templateId}`, {
      method: "DELETE",
    })
  }

  async adminTestMailTemplate(templateId: string) {
    return this.request<{ sent: true }>(`/api/admin/mail/templates/${templateId}/test`, {
      method: "POST",
    })
  }

  async adminGetWebhooks() {
    return this.request<{ webhooks: any[] }>("/api/admin/webhooks")
  }

  async adminCreateWebhook(data: any) {
    return this.request<{ id: string }>("/api/admin/webhooks", {
      method: "POST",
      body: data,
    })
  }

  async adminUpdateWebhook(webhookId: string, data: any) {
    return this.request<{ updated: true }>(`/api/admin/webhooks/${webhookId}`, {
      method: "PUT",
      body: data,
    })
  }

  async adminDeleteWebhook(webhookId: string) {
    return this.request<{ deleted: boolean }>(`/api/admin/webhooks/${webhookId}`, {
      method: "DELETE",
    })
  }

  async adminTestWebhook(webhookId: string) {
    return this.request<{ sent: true }>(`/api/admin/webhooks/${webhookId}/test`, {
      method: "POST",
    })
  }

  async adminGetAnalytics() {
    return this.request<any>("/api/admin/analytics")
  }

  // ==================== Sponsor Endpoints ====================

  async sponsorGetBooth() {
    return this.request<{ booth: any }>("/api/sponsor/booth")
  }

  async sponsorCreateBooth(data: any) {
    return this.request<{ id: string }>("/api/sponsor/booth", {
      method: "POST",
      body: data,
    })
  }

  async sponsorUpdateBooth(boothId: string, data: any) {
    return this.request<{ updated: true }>(`/api/sponsor/booth/${boothId}`, {
      method: "PUT",
      body: data,
    })
  }

  async sponsorDeleteBooth(boothId: string) {
    return this.request<{ deleted: boolean }>(`/api/sponsor/booth/${boothId}`, {
      method: "DELETE",
    })
  }

  async sponsorGetPrizes() {
    return this.request<{ prizes: any[] }>("/api/sponsor/prizes")
  }

  async sponsorCreatePrize(data: any) {
    return this.request<{ id: string }>("/api/sponsor/prizes", {
      method: "POST",
      body: data,
    })
  }

  async sponsorUpdatePrize(prizeId: string, data: any) {
    return this.request<{ updated: true }>(`/api/sponsor/prizes/${prizeId}`, {
      method: "PUT",
      body: data,
    })
  }

  async sponsorDeletePrize(prizeId: string) {
    return this.request<{ deleted: boolean }>(`/api/sponsor/prizes/${prizeId}`, {
      method: "DELETE",
    })
  }

  async sponsorSelectWinner(prizeId: string, teamId: string) {
    return this.request<{ selected: true }>(`/api/sponsor/prizes/${prizeId}/winner`, {
      method: "POST",
      body: { team_id: teamId },
    })
  }

  async sponsorGetSubmissions() {
    return this.request<{ submissions: any[] }>("/api/sponsor/submissions")
  }

  async sponsorApproveSubmission(submissionId: string) {
    return this.request<{ approved: true }>(`/api/sponsor/submissions/${submissionId}/approve`, {
      method: "POST",
    })
  }

  async sponsorRejectSubmission(submissionId: string) {
    return this.request<{ rejected: true }>(`/api/sponsor/submissions/${submissionId}/reject`, {
      method: "POST",
    })
  }

  // Discord Bot
  async discordInitiateLink(discordUserId: string) {
    return this.request<{ token: string; expires_in: number }>("/api/discord-bot/link/initiate", {
      method: "POST",
      body: { discord_user_id: discordUserId },
    })
  }

  async discordConfirmLink(token: string, openhackUserId: string) {
    return this.request<{ status: string }>("/api/discord-bot/link/confirm", {
      method: "POST",
      body: { token, openhack_user_id: openhackUserId },
    })
  }

  async discordGetLinkStatus(discordId?: string, openhackId?: string) {
    const params = new URLSearchParams()
    if (discordId) params.set("discord_id", discordId)
    if (openhackId) params.set("openhack_id", openhackId)
    return this.request<{ linked: boolean; discord_user_id?: string; openhack_user_id?: string }>(
      `/api/discord-bot/link/status?${params.toString()}`
    )
  }

  async discordGetBotStatus() {
    return this.request<{ status: string; bot_connected: boolean; guild_id: string; latency_ms: number }>(
      "/api/discord-bot/status"
    )
  }
}

export const api = new ApiClient()
