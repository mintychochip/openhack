import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

const gatewayErrorRate = new Rate('gateway_errors');
const authErrorRate = new Rate('auth_errors');
const coreErrorRate = new Rate('core_errors');
const judgingErrorRate = new Rate('judging_errors');
const leaderboardErrorRate = new Rate('leaderboard_errors');
const mailErrorRate = new Rate('mail_errors');
const notifyErrorRate = new Rate('notify_errors');
const aiErrorRate = new Rate('ai_errors');
const analyticsErrorRate = new Rate('analytics_errors');
const sponsorsErrorRate = new Rate('sponsors_errors');
const mediaErrorRate = new Rate('media_errors');

// When using port-forward, each service is on localhost at its service port.
// When using ingress, set GATEWAY_URL to the ingress host.
// The orchestrator sets these env vars based on how it connects.
const GATEWAY_URL = __ENV.GATEWAY_URL || 'http://localhost:8000';
const AUTH_URL = __ENV.AUTH_URL || 'http://localhost:3001';
const CORE_URL = __ENV.CORE_URL || 'http://localhost:3002';
const JUDGING_URL = __ENV.JUDGING_URL || 'http://localhost:3003';
const LEADERBOARD_URL = __ENV.LEADERBOARD_URL || 'http://localhost:3004';
const MAIL_URL = __ENV.MAIL_URL || 'http://localhost:3005';
const NOTIFY_URL = __ENV.NOTIFY_URL || 'http://localhost:3006';
const AI_URL = __ENV.AI_URL || 'http://localhost:3007';
const ANALYTICS_URL = __ENV.ANALYTICS_URL || 'http://localhost:3008';
const SPONSORS_URL = __ENV.SPONSORS_URL || 'http://localhost:3009';
const MEDIA_URL = __ENV.MEDIA_URL || 'http://localhost:3010';

export const options = {
    stages: [
        { duration: '2m', target: 20 },
        { duration: '5m', target: 50 },
        { duration: '2m', target: 20 },
        { duration: '1m', target: 0 },
    ],
    thresholds: {
        errors: ['rate<0.15'],
        http_req_duration: ['p(95)<3000'],
        http_req_failed: ['rate<0.15'],
    },
};

const TEST_EMAIL = `memtest_${Date.now()}@example.com`;
const TEST_PASSWORD = 'MemTestPass123!';

function makeAuthHeaders(token) {
    return {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
    };
}

export function setup() {
    const regRes = http.post(`${GATEWAY_URL}/api/auth/register`, JSON.stringify({
        email: TEST_EMAIL,
        password: TEST_PASSWORD,
        name: 'Memory Test User',
    }), { headers: { 'Content-Type': 'application/json' } });

    check(regRes, { 'setup: register 200 or 409': (r) => r.status === 200 || r.status === 409 });

    const loginRes = http.post(`${GATEWAY_URL}/api/auth/login`, JSON.stringify({
        email: TEST_EMAIL,
        password: TEST_PASSWORD,
    }), { headers: { 'Content-Type': 'application/json' } });

    const loginOk = check(loginRes, { 'setup: login 200': (r) => r.status === 200 });
    if (!loginOk) {
        console.error('Login failed, aborting setup');
        return { authToken: '' };
    }

    const body = JSON.parse(loginRes.body);
    console.log('Setup complete — token obtained');
    return { authToken: body.access_token || '' };
}

export default function (data) {
    const token = data.authToken || '';
    const headers = makeAuthHeaders(token);

    // ── Phase A: Gateway-proxied traffic (realistic user flow) ──
    group('gateway-proxied', () => {
        const res = http.get(`${GATEWAY_URL}/health`);
        check(res, { 'gateway health 200': (r) => r.status === 200 });
        gatewayErrorRate.add(res.status >= 400);

        if (token) {
            const me = http.get(`${GATEWAY_URL}/api/auth/me`, { headers });
            check(me, { 'gateway auth/me 200': (r) => r.status === 200 || r.status === 401 });
            gatewayErrorRate.add(me.status >= 500);

            const teams = http.get(`${GATEWAY_URL}/api/core/teams`, { headers });
            check(teams, { 'gateway teams 200': (r) => r.status === 200 || r.status === 401 });
            gatewayErrorRate.add(teams.status >= 500);

            const projects = http.get(`${GATEWAY_URL}/api/core/projects`, { headers });
            check(projects, { 'gateway projects 200': (r) => r.status === 200 || r.status === 401 });
            gatewayErrorRate.add(projects.status >= 500);

            const rubrics = http.get(`${GATEWAY_URL}/api/judging/rubrics`, { headers });
            check(rubrics, { 'gateway rubrics 200': (r) => r.status === 200 || r.status === 401 });
            rubrics.status >= 500 && gatewayErrorRate.add(true);

            const leaderboard = http.get(`${GATEWAY_URL}/api/leaderboard/rankings`, { headers });
            check(leaderboard, { 'gateway leaderboard 200': (r) => r.status === 200 || r.status === 401 });
            gatewayErrorRate.add(leaderboard.status >= 500);
        }
    });

    // ── Phase B: Direct service hits (exercise each service independently) ──

    group('auth-direct', () => {
        const res = http.get(`${AUTH_URL}/health`);
        check(res, { 'auth health 200': (r) => r.status === 200 });
        authErrorRate.add(res.status >= 400);

        const ready = http.get(`${AUTH_URL}/ready`);
        check(ready, { 'auth ready 200': (r) => r.status === 200 || r.status === 503 });
        authErrorRate.add(ready.status >= 500);

        if (token) {
            const me = http.get(`${AUTH_URL}/me`, { headers });
            check(me, { 'auth /me responds': (r) => r.status < 500 });
            authErrorRate.add(me.status >= 500);
        }
    });

    group('core-direct', () => {
        const res = http.get(`${CORE_URL}/health`);
        check(res, { 'core health 200': (r) => r.status === 200 });
        coreErrorRate.add(res.status >= 400);

        const ready = http.get(`${CORE_URL}/ready`);
        check(ready, { 'core ready 200': (r) => r.status === 200 || r.status === 503 });
        coreErrorRate.add(ready.status >= 500);

        if (token) {
            const teams = http.get(`${CORE_URL}/teams`, { headers });
            check(teams, { 'core teams responds': (r) => r.status < 500 });
            coreErrorRate.add(teams.status >= 500);

            const events = http.get(`${CORE_URL}/events`, { headers });
            check(events, { 'core events responds': (r) => r.status < 500 });
            coreErrorRate.add(events.status >= 500);
        }
    });

    group('judging-direct', () => {
        const res = http.get(`${JUDGING_URL}/health`);
        check(res, { 'judging health 200': (r) => r.status === 200 });
        judgingErrorRate.add(res.status >= 400);

        if (token) {
            const rubrics = http.get(`${JUDGING_URL}/rubrics`, { headers });
            check(rubrics, { 'judging rubrics responds': (r) => r.status < 500 });
            judgingErrorRate.add(rubrics.status >= 500);

            const dashboard = http.get(`${JUDGING_URL}/dashboard`, { headers });
            check(dashboard, { 'judging dashboard responds': (r) => r.status < 500 });
            judgingErrorRate.add(dashboard.status >= 500);
        }
    });

    group('leaderboard-direct', () => {
        const res = http.get(`${LEADERBOARD_URL}/health`);
        check(res, { 'leaderboard health 200': (r) => r.status === 200 });
        leaderboardErrorRate.add(res.status >= 400);

        const ready = http.get(`${LEADERBOARD_URL}/ready`);
        check(ready, { 'leaderboard ready 200': (r) => r.status === 200 || r.status === 503 });
        leaderboardErrorRate.add(ready.status >= 500);

        if (token) {
            const rankings = http.get(`${LEADERBOARD_URL}/rankings`, { headers });
            check(rankings, { 'leaderboard rankings responds': (r) => r.status < 500 });
            leaderboardErrorRate.add(rankings.status >= 500);

            const stats = http.get(`${LEADERBOARD_URL}/stats`, { headers });
            check(stats, { 'leaderboard stats responds': (r) => r.status < 500 });
            leaderboardErrorRate.add(stats.status >= 500);
        }
    });

    group('mail-direct', () => {
        const res = http.get(`${MAIL_URL}/health`);
        check(res, { 'mail health 200': (r) => r.status === 200 || r.status === 503 });
        mailErrorRate.add(res.status >= 500);
    });

    group('notify-direct', () => {
        const res = http.get(`${NOTIFY_URL}/health`);
        check(res, { 'notify health 200': (r) => r.status === 200 || r.status === 503 });
        notifyErrorRate.add(res.status >= 500);
    });

    group('ai-direct', () => {
        const res = http.get(`${AI_URL}/health`);
        check(res, { 'ai health 200': (r) => r.status === 200 });
        aiErrorRate.add(res.status >= 400);
    });

    group('analytics-direct', () => {
        const res = http.get(`${ANALYTICS_URL}/health`);
        check(res, { 'analytics health 200': (r) => r.status === 200 });
        analyticsErrorRate.add(res.status >= 400);
    });

    group('sponsors-direct', () => {
        const res = http.get(`${SPONSORS_URL}/health`);
        check(res, { 'sponsors health 200': (r) => r.status === 200 });
        sponsorsErrorRate.add(res.status >= 400);

        const ready = http.get(`${SPONSORS_URL}/ready`);
        check(ready, { 'sponsors ready 200': (r) => r.status === 200 || r.status === 503 });
        sponsorsErrorRate.add(ready.status >= 500);
    });

    group('media-direct', () => {
        const res = http.get(`${MEDIA_URL}/health`);
        check(res, { 'media health 200': (r) => r.status === 200 });
        mediaErrorRate.add(res.status >= 400);
    });

    sleep(1);
}

export function teardown(data) {
    console.log('Memory test k6 run complete.');
}
