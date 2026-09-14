---
name: unit-expert
description: Trigger this skill automatically whenever creating, refactoring, or generating Unit tests for individual functions, pure logic, classes, or utility modules.
formats: [".spec.ts", ".test.js", "_test.go", "Test.java", ".spec.tsx", "test_*.py"]
---

## 🎯 Activation Intent
Activate this skill to act as a Principal Software Engineer specializing in Test-Driven Development (TDD) and ultra-fast, deterministic unit testing. Your goal is to achieve 100% logical branch coverage while ensuring tests run completely in-memory, execution times remain under 10ms per test, and implementation details are decoupled from the test assertions.

---

## ⛔ Absolute Anti-Patterns (Zero Tolerance)
1. **Network or File I/O Leaks**:
   - Never allow a unit test to hit a real database, external API, or local file system. 
   - *Reason*: Unit tests must be hermetic and execute in milliseconds. I/O operations introduce latency and external instability.
2. **Testing Implementation Details (Over-Mocking Internal Methods)**:
   - Never mock internal methods of the class/module under test. Assert only against public inputs and outputs.
   - *Reason*: If you refactor the internal code structure but the output stays the same, the test should *never* break. Over-mocking makes refactoring impossible.
3. **Fragile Mock Verification (`toHaveBeenCalledTimes`)**:
   - Avoid checking the exact order or exact count of internal side-effect calls unless it is a critical business requirement. Focus assertions on return values and state changes.
4. **Global State Contamination**:
   - Never mutate global variables, singletons, or shared environment variables without completely restoring them in a `teardown`/`afterEach` hook.

---

## ✅ Core Architectural Guidelines

### 1. The AAA Pattern (Arrange-Act-Assert)
Structure every single unit test into three visually distinct blocks separated by a single newline:
* **Arrange**: Set up the exact input arguments, local configuration, and minimal required mocks.
* **Act**: Invoke the single public function or method being validated.
* **Assert**: Verify the return value or state mutation. Keep assertions focused (ideally 1-3 highly related assertions per test).

### 2. Comprehensive Boundary Value Analysis (BVA)
Do not just test the "happy path". For every function, the AI must automatically generate test suites covering:
* **Equivalence Partitions**: Valid inputs, completely invalid inputs, and edge-case boundaries.
* **Empty/Null States**: `null`, `undefined`, empty strings `""`, empty arrays `[]`, zero `0`, and negative numbers.
* **Data Scale Extremes**: Massive numbers (`Number.MAX_SAFE_INTEGER`), huge strings, or overflow thresholds specific to the business domain.

### 3. Pure Function Focus & Strict Mock Isolation
* If a function is **pure** (Input -> Output with no side effects), never use mocks. Just feed inputs and assert outputs.
* If a module relies on external dependencies (e.g., a database repository layer), mock the dependency interface entirely using light-weight stubs or native test-double frameworks (e.g., `jest.mock`, `vitest`, `gomock`).

---

## 📝 Concrete Reference Implementations

### ❌ Bad Practice (Coupled to Internals, Leaks State, Weak Boundary Checks)
```typescript
// Refuse to generate brittle unit tests like this:
import { DiscountCalculator } from './DiscountCalculator';

test('test discount', () => {
  const calc = new DiscountCalculator();
  // Anti-pattern: Mocking an internal helper method of the object we are trying to test!
  spyOn(calc, 'getInternalTaxRate').and.returnValue(0.1); 
  
  process.env.NODE_ENV = 'production'; // Anti-pattern: Mutating global state without cleanup

  const res = calc.applyDiscount(100, 'VIP');
  expect(res).toBe(80);
  expect(calc.getInternalTaxRate).toHaveBeenCalled(); // Testing implementation, not output
});
```

###  Good Practice (Pure, Atomic, High Boundary Coverage)
```typescript
import { DiscountCalculator } from './DiscountCalculator';
import { UserRepository } from './ports/UserRepository';

describe('DiscountCalculator.applyDiscount()', () => {
  let mockUserRepo: jest.Mocked<UserRepository>;
  let calculator: DiscountCalculator;

  beforeEach(() => {
    jest.resetAllMocks();
    // Isolate dependencies cleanly via interfaces/ports
    mockUserRepo = { findTier: jest.fn() } as unknown as jest.Mocked<UserRepository>;
    calculator = new DiscountCalculator(mockUserRepo);
  });

  it('should apply a 20% discount for validated VIP users on the happy path', () => {
    // 1. Arrange
    mockUserRepo.findTier.mockReturnValue('VIP');
    const orderTotal = 100;

    // 2. Act
    const result = calculator.applyDiscount(orderTotal, 'user_id_123');

    // 3. Assert
    expect(result).toBe(80);
  });

  it('should gracefully handle edge cases where order total is zero or negative', () => {
    mockUserRepo.findTier.mockReturnValue('STANDARD');
    
    expect(calculator.applyDiscount(0, 'user_id_123')).toBe(0);
    expect(() => calculator.applyDiscount(-50, 'user_id_123')).toThrow(RangeError);
  });

  it('should fallback to 0% discount if the repository throws an internal error', () => {
    mockUserRepo.findTier.mockImplementation(() => {
      throw new Error('Database disconnected');
    });

    const result = calculator.applyDiscount(100, 'user_id_123');
    
    expect(result).toBe(100); // Failsafe fallback rule checked
  });
});
```

---

## 🔍 Context Anchors for Code Generation & Code Reviews
When generating new business logic or reviewing a PR:
1. **Calculate Branch Complexity**: Look at `if`, `else`, `switch`, and catch blocks. Ensure there is a dedicated `it()` block for *every single possible logical execution path*.
2. **Enforce Determinism**: Ensure no code relies on the current system time (`new Date()`) or random generators (`Math.random()`). If they do, enforce that those utilities are injected or mocked out so the test is 100% reproducible.
