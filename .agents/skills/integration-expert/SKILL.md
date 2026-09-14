---
name: integration-expert
description: Trigger this skill automatically when writing or refactoring tests that verify interactions between multiple systems, databases, queues, or third-party APIs.
formats: [".spec.ts", ".test.js", "_integration_test.go", "IT.java", "test_integration.py"]
---

## 🎯 Activation Intent
Activate this skill to write high-confidence integration tests. Your goal is to verify that independent modules, networks, and data stores work together perfectly under realistic conditions, without testing pure business logic or end-to-end user flows.

---

## ⛔ Absolute Anti-Patterns (Zero Tolerance)
1. **Mocking internal databases or network clients**:
   - Never use standard code mocks (`jest.mock`, `Mockito.mock`) for your own database repositories or external API network clients in this mode.
   - *Reason*: Integration tests must execute actual network/database drivers to catch real schema mismatches, query errors, and serialization bugs.
2. **Hardcoded system ports**:
   - Never bind test containers or local test servers to fixed static ports (e.g., forcing a PostgreSQL container to use port `5432` on the host).
   - *Reason*: Destroys parallel execution in CI/CD environments and causes instant port-collision failures.
3. **Leaky teardowns (Dirty state)**:
   - Never leave tables populated, queues full, or storage buckets dirty after a test finishes.
   - *Reason*: State pollution causes unpredictable cascade failures in subsequent tests.

---

## ✅ Core Architectural Guidelines

### 1. Ephemeral Dependency Lifecycles
* **On-the-fly Infrastructure**: Spin up real infrastructure (PostgreSQL, Redis, Kafka, WireMock) programmatically using modern test-container libraries or native local environment isolation.
* **Orchestration**: Start external dependencies once per test suite using `beforeAll` / `setupSuite` hooks to keep execution efficient, but completely destroy them during `afterAll`.

### 2. Sandbox Data Isolation
* **Schema Migration**: Always run your system's actual migration scripts against the freshly spun-up database container before executing tests.
* **Truncation Over Re-creation**: Instead of destroying and rebuilding the container between individual tests, execute a fast table truncation or database cleanup script in `afterEach`.

### 3. External API Verification (WireMock/Hoverfly)
* **Contract Simulation**: When integrating with third-party HTTP/gRPC services, intercept the network layer using a dedicated mocking server (e.g., WireMock) rather than mocking the internal code.
* **Fault Injection**: Test how the application handles external dependency failures by deliberately simulating slow network timeouts (e.g., 504 Gateway Timeout) and bad payloads.

---

## 📝 Concrete Reference Implementations

### ❌ Bad Practice (Mocking the wrong layer, hardcoded ports)
```typescript
// Refuse to generate pseudo-integration tests like this:
import { UserService } from './UserService';
import { dbClient } from './db';

jest.mock('./db'); // Anti-pattern: Mocking the database client inside an integration test!

test('should save user', async () => {
  dbClient.query.mockResolvedValue({ id: 1 }); // Does not test real SQL or schema matching
  
  const service = new UserService();
  const user = await service.createUser({ name: 'Alice' });
  expect(user.id).toBe(1);
});
```

###  Good Practice (Real infrastructure interaction, dynamic configuration)
```typescript
import { UserService } from './UserService';
import { DatabasePool } from './DatabasePool';
import { PostgreSqlContainer } from '@testcontainers/postgresql';

describe('UserService Integration with PostgreSQL', () => {
  let container: PostgreSqlContainer;
  let dbPool: DatabasePool;
  let userService: UserService;

  beforeAll(async () => {
    // 1. Arrange: Spin up clean, real infrastructure dynamically
    container = await new PostgreSqlContainer().start();
    dbPool = new DatabasePool({ url: container.getConnectionString() });
    
    // Run actual migrations against the container
    await dbPool.runMigrations(); 
    userService = new UserService(dbPool);
  });

  afterAll(async () => {
    await dbPool.close();
    await container.stop(); // Safe, clean teardown
  });

  afterEach(async () => {
    await dbPool.truncateAllTables(); // Isolate test state between runs
  });

  it('should successfully commit a user to the database and retrieve it with active status', async () => {
    // 2. Act: Execute real query compilation and database side-effects
    const savedUser = await userService.createUser({ name: 'Alice', email: 'alice@example.internal' });
    const fetchedUser = await userService.findUserById(savedUser.id);

    // 3. Assert: Verify the operational contract between code and data store
    expect(fetchedUser).toBeDefined();
    expect(fetchedUser.email).toBe('alice@example.internal');
    expect(fetchedUser.createdAt).toBeInstanceOf(Date); // Verifies driver serialization
  });
});
```

---

## 🔍 Context Anchors for Code Generation & Reviews
When writing or reviewing integration setups:
1. **Verify Boundary Context**: Check if you are crossing a system boundary (Network, Process, File System). If yes, ensure a real or simulated network/I/O interface is being tested, not a fake language object.
2. **Check for Async Race Conditions**: When testing message queues (Kafka, RabbitMQ), make sure the test includes an asynchronous polling mechanism (e.g., `await waitFor()`) rather than a hardcoded wait time to catch the emitted event.
