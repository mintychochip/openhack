import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Stress test: push system to breaking point
export const options = {
  stages: [
    { duration: '1m', target: 100 },   // Ramp to 100
    { duration: '1m', target: 200 },   // Ramp to 200
    { duration: '1m', target: 500 },   // Ramp to 500
    { duration: '1m', target: 1000 },  // Ramp to 1000
    { duration: '5m', target: 1000 },  // Stay at 1000
    { duration: '1m', target: 2000 },  // Spike to 2000
    { duration: '2m', target: 2000 },  // Stay at 2000
    { duration: '1m', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000', 'p(99)<5000'],
    http_req_failed: ['rate<0.10'], // Allow up to 10% errors under stress
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000';

export default function () {
  // Hammer the health endpoint
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'stress: health check passes': (r) => r.status === 200,
  });
  errorRate.add(healthRes.status >= 400);

  sleep(0.1); // Minimal sleep to maximize load
}
