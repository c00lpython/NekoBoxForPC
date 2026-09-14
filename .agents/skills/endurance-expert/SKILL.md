---
name: endurance-expert
description: Trigger this skill automatically when generating scripts, scenarios, or configurations to test system stability, resource leaks, and performance degradation under prolonged continuous load.
formats: [".js", ".ts", ".jmx", ".scala", "locustfile.py", "k6", "gatling"]
---

## 🎯 Activation Intent
Activate this skill to act as a Performance Engineer specializing in soak, longevity, and endurance testing. Your goal is to design automated testing scenarios that simulate continuous, stable user load over extended periods to discover memory leaks, resource exhaustion, database connection pool depletion, and slow performance degradation (software rot).

---

## ⛔ Absolute Anti-Patterns (Zero Tolerance)
1. **The "Local Storm" Effect (Resource Starvation of the Generator)**:
   - Never write resource-heavy script logic (like parsing massive JSON objects inside virtual user loops or console logging every HTTP request).
   - *Reason*: This exhausts the CPU/RAM of the load generator itself, leading to false-positive timeouts and invalid metrics.
2. **Aggressive Spike Load**:
   - Never inject sudden, massive bursts of traffic in an endurance scenario.
   - *Reason*: Endurance testing is designed to find gradual deterioration under a stable, sustainable plateau. Sudden spikes convert the test into a Stress Test, masking long-term memory leaks.
3. **Omitting Think Time (Static Continuous Hammering)**:
   - Never let virtual users (VUs) fire consecutive requests without random pacing pauses.
   - *Reason*: Real users do not click buttons instantly. Zero think time simulates a DDoS attack, which forces web servers to block traffic or fail immediately instead of exposing long-term database lock issues.

---

## ✅ Core Architectural Guidelines

### 1. The Extended Soak Profile
* **Flat Load Plateau**: Design configurations with a distinct three-stage lifecycle: a gentle ramp-up (5–10% of total duration), a long, flat execution plateau (80-90% of total duration), and a controlled ramp-down.
* **Metric Baselines**: Force assertions to compare metrics from the *first 10 minutes* of the plateau directly against the *last 10 minutes* to explicitly catch progressive system degradation.

### 2. Pacing and Randomization (Think Time)
* **Poisoneous Patterns**: Use randomized intervals (e.g., Gaussian or uniform distributions between 1 and 3 seconds) for execution pauses to prevent synchronous "micro-bursting" waves of VUs hitting the infrastructure at the exact same millisecond.
* **Dynamic Test Data**: Parametrize payloads using unique data blocks (CSV feeders, UUID generators) to bypass proxy/CDN caching and hit the actual data layers continuously.

### 3. Service Level Objectives (SLOs) as Code
* **Error Rate Thresholds**: Enforce strict error rate thresholds (typically `rate < 0.001` or less than 0.1% failures over the entire lifecycle).
* **Percentile Anchors**: Assert against the 95th or 99th percentile (`p(95)` / `p(99)`) of response times rather than the average. If the `p(95)` chart line shows a constant upward slope, fail the test due to degradation.

---

## 📝 Concrete Reference Implementations

### ❌ Bad Practice (No pacing, no threshold validation, heavy console logging)
```javascript
// Refuse to generate inefficient, unmonitored load loops like this:
import http from 'k6/http';

export default function () {
  // Anti-pattern: No think time / pacing. Loops instantly.
  const res = http.get('https://target.internal');
  
  // Anti-pattern: Heavy I/O logging during high-concurrency load
  console.log('Response body: ' + res.body); 
  
  // Anti-pattern: No automated performance degradation thresholds defined
}
```

###  Good Practice (Paced, randomized, strict percentile-degradation checks)
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  // 1. Arrange: Defined extended plateau for long-duration execution
  stages: [
    { duration: '10m', target: 200 },  // Controlled Ramp-up
    { duration: '2h',  target: 200 },  // Endurance / Soak testing flat plateau
    { duration: '5m',  target: 0 },    // Controlled Ramp-down
  ],
  // 3. Assert: Structural SLOs that catch progressive degradation over time
  thresholds: {
    // 95% of all requests across the entire test must resolve under 400ms
    'http_req_duration': ['p(95)<400'], 
    // The error rate must remain strictly below 0.1%
    'http_req_failed': ['rate<0.001'],  
  },
};

export default function () {
  // 2. Act: Execute real endpoints using realistic data parametrization
  const headers = { 'Content-Type': 'application/json' };
  const res = http.get('https://target.internal', { headers });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'content is valid': (r) => r.json().items.length > 0,
  });

  // Pacing: Mimic organic user behavior with a 1-3 second randomized pause
  sleep(Math.random() * 2 + 1);
}
```

---

## 🔍 Context Anchors for Code Generation & Reviews
When creating or checking endurance suites:
1. **Leak Detection Alignment**: Verify that the environment running this test has memory/CPU monitoring attached (e.g., Prometheus/Grafana dashboard hooks). Ensure the script outputs clean metric tracking tags (`tags: { name: 'checkout-flow' }`).
2. **Connection Leak Auditing**: Ensure that open sockets or connections created by virtual user routines are explicitly closed or pooled correctly to avoid running out of local file descriptors on the host runner.
