import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const checkRate = new Rate('checks_passed');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 50 },   // Ramp up to 50 users
    { duration: '5m', target: 50 },   // Stay at 50 users
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 200 },  // Ramp up to 200 users
    { duration: '5m', target: 200 },  // Stay at 200 users
    { duration: '2m', target: 500 },  // Spike to 500 users
    { duration: '3m', target: 500 },  // Stay at 500 users
    { duration: '2m', target: 0 },    // Ramp down to 0
  ],
  thresholds: {
    http_req_duration: ['p(50)<500', 'p(95)<1000', 'p(99)<2000'], // 50% < 500ms, 95% < 1s, 99% < 2s
    http_req_failed: ['rate<0.05'], // Error rate < 5%
    checks_passed: ['rate>0.95'], // 95% checks pass
    errors: ['rate<0.05'], // 5% error threshold
  },
};

// Test data
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000';
const TEST_EMAIL = `test_${Date.now()}@example.com`;
const TEST_PASSWORD = 'TestPassword123!';

let authToken = '';
let userId = '';
let teamId = '';
let projectId = '';

// Initial registration and login
export function setup() {
  console.log('Setting up test data...');

  // Register new user
  const registerRes = http.post(`${BASE_URL}/api/auth/register`, JSON.stringify({
    email: TEST_EMAIL,
    password: TEST_PASSWORD,
    name: 'Load Test User',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(registerRes, {
    'setup: registration status is 200 or 409': (r) => r.status === 200 || r.status === 409,
  });

  // Login
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
    email: TEST_EMAIL,
    password: TEST_PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(loginRes, {
    'setup: login status is 200': (r) => r.status === 200,
  });

  const body = JSON.parse(loginRes.body);
  authToken = body.access_token;
  userId = body.user_id;

  console.log('Setup complete. Token obtained.');

  return { authToken, userId };
}

// Virtual user function
export default function (data) {
  const { authToken } = data;

  const headers = {
    'Authorization': `Bearer ${authToken}`,
    'Content-Type': 'application/json',
  };

  // Test 1: Health check
  testHealthCheck();

  // Test 2: Auth endpoints
  testAuthEndpoints(headers);

  // Test 3: Core endpoints (teams, projects)
  testCoreEndpoints(headers);

  // Test 4: Judging endpoints
  testJudgingEndpoints(headers);

  // Test 5: Leaderboard endpoints
  testLeaderboardEndpoints(headers);

  sleep(1); // Wait 1 second between iterations
}

function testHealthCheck() {
  const res = http.get(`${BASE_URL}/health`);
  const passed = check(res, {
    'health check: status is 200': (r) => r.status === 200,
    'health check: response time < 100ms': (r) => r.timings.duration < 100,
  });
  errorRate.add(res.status >= 400);
  checkRate.add(passed);
}

function testAuthEndpoints(headers) {
  // Get current user profile
  const profileRes = http.get(`${BASE_URL}/api/auth/me`, { headers });
  const profilePassed = check(profileRes, {
    'profile: status is 200': (r) => r.status === 200,
    'profile: response time < 200ms': (r) => r.timings.duration < 200,
  });
  errorRate.add(profileRes.status >= 400);
  checkRate.add(profilePassed);

  sleep(0.5);
}

function testCoreEndpoints(headers) {
  // List teams
  const teamsRes = http.get(`${BASE_URL}/api/core/teams`, { headers });
  const teamsPassed = check(teamsRes, {
    'teams list: status is 200': (r) => r.status === 200,
    'teams list: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(teamsRes.status >= 400);
  checkRate.add(teamsPassed);

  // List projects
  const projectsRes = http.get(`${BASE_URL}/api/core/projects`, { headers });
  const projectsPassed = check(projectsRes, {
    'projects list: status is 200': (r) => r.status === 200,
    'projects list: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(projectsRes.status >= 400);
  checkRate.add(projectsPassed);

  // List events
  const eventsRes = http.get(`${BASE_URL}/api/core/events`, { headers });
  const eventsPassed = check(eventsRes, {
    'events list: status is 200': (r) => r.status === 200,
    'events list: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(eventsRes.status >= 400);
  checkRate.add(eventsPassed);

  sleep(0.5);
}

function testJudgingEndpoints(headers) {
  // List rubrics
  const rubricsRes = http.get(`${BASE_URL}/api/judging/rubrics`, { headers });
  const rubricsPassed = check(rubricsRes, {
    'rubrics list: status is 200': (r) => r.status === 200,
    'rubrics list: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(rubricsRes.status >= 400);
  checkRate.add(rubricsPassed);

  // Get judging dashboard
  const dashboardRes = http.get(`${BASE_URL}/api/judging/dashboard`, { headers });
  const dashboardPassed = check(dashboardRes, {
    'dashboard: status is 200': (r) => r.status === 200,
    'dashboard: response time < 500ms': (r) => r.timings.duration < 500,
  });
  errorRate.add(dashboardRes.status >= 400);
  checkRate.add(dashboardPassed);

  sleep(0.5);
}

function testLeaderboardEndpoints(headers) {
  // Get leaderboard
  const leaderboardRes = http.get(`${BASE_URL}/api/leaderboard/rankings`, { headers });
  const leaderboardPassed = check(leaderboardRes, {
    'leaderboard: status is 200': (r) => r.status === 200,
    'leaderboard: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(leaderboardRes.status >= 400);
  checkRate.add(leaderboardPassed);

  // Get stats
  const statsRes = http.get(`${BASE_URL}/api/leaderboard/stats`, { headers });
  const statsPassed = check(statsRes, {
    'stats: status is 200': (r) => r.status === 200,
    'stats: response time < 300ms': (r) => r.timings.duration < 300,
  });
  errorRate.add(statsRes.status >= 400);
  checkRate.add(statsPassed);

  sleep(0.5);
}

// Teardown (optional cleanup)
export function teardown(data) {
  console.log('Load test completed.');
  // Optionally delete test user here
}
