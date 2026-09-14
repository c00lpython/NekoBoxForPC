---
name: e2e-expert
description: Trigger this skill automatically whenever generating, refactoring, or debugging End-to-End (E2E) automation tests for Web, Mobile, or Desktop UIs.
formats: [".spec.ts", ".test.js", ".feature", "cy.js", ".py", "BasePage"]
---

## 🎯 Activation Intent
Activate this skill to act as a Staff QA Automation Engineer. Your goal is to write E2E tests that are execution-stable, completely deterministic, highly maintainable, and strictly decoupled from volatile UI implementation details.

---

## ⛔ Absolute Anti-Patterns (Zero Tolerance)
1. **Time-Based Hardcoded Waits**: 
   - Never use `sleep()`, `waitForTimeout()`, or `Thread.sleep()`. 
   - *Reason*: This is the primary root cause of flaky tests and bloated CI/CD pipeline runtimes.
2. **Brittle Structural Selectors**: 
   - Never target elements using strict DOM/hierarchy layouts (e.g., `div > div > span:nth-child(3)` or full absolute XPaths).
   - *Reason*: Minor UI redesigns or framework updates will instantly break the entire test suite.
3. **Implicit Test Interdependency**: 
   - Never let Test B rely on the state, side-effects, or data mutations left behind by Test A. 
   - *Reason*: Prevents parallel execution and makes isolated debugging impossible.
4. **Leaking Secrets**: 
   - Never hardcode user credentials, API keys, or environment URLs inside test blocks.

---

## ✅ Core Architectural Guidelines

### 1. Robust Element Targeting (Semantic-First)
Always locate interactive elements by leveraging user-facing semantics or explicit testing attributes, ranked in this strict order of priority:
* **Priority 1: Accessible Roles & Labels** (e.g., `getByRole('button', { name: 'Submit' })`, `getByLabel()`). Reflects how real users (and assistive technologies) perceive the page.
* **Priority 2: Text Content** (e.g., `getByText()`). Ideal for static, stable UI components.
* **Priority 3: Dedicated QA Attributes** (e.g., `data-testid`, `data-qa`, `accessibilityIdentifier`). Use this when semantic attributes are non-unique or dynamic.

### 2. State Isolation & Intelligent Authentication
* **Hermetic Environments**: Every test scenario must start with a clean state. Use API calls or database seeds in setup hooks (`beforeAll`/`beforeEach`) to instantly forge required preconditions.
* **Session Re-use**: Bypass repetitive UI login forms. Authenticate once via an API or background script, dump the state (tokens/cookies/localStorage), and inject it directly into the browser context for subsequent tests.

### 3. Execution Stability via Smart Assertions
* **Deterministic Retries & Auto-Waiting**: Utilize assertions that automatically poll the application state (e.g., web-first assertions). 
* **State Readiness Hooks**: Before interacting with elements, assert that the application has fully loaded (e.g., spinners vanished, API responses resolved, network idle).

### 4. Code Abstraction (Maintainability Model)
* **Encapsulate UI Logic**: Isolate element selectors, component interactions, and page sub-routines away from the test file. Use Page Object Models (POM), Screenplay Patterns, or Component-Driven architectures.
* **Self-Documenting Steps**: Wrap dense interaction flows into high-level business functions (e.g., `page.checkoutCartWithPromoCode('SUMMER26')`).

---

## 📝 Concrete Reference Implementations

### ❌ Bad Practice (Flaky, Brittle, Untenably Interdependent)
```typescript
// Refuse to generate code like this:
test('checkout test', async ({ page }) => {
  await page.goto('https://my-app.internal');
  await page.locator('#input-1').fill('admin'); // Brittle ID
  await page.locator('input').last().fill('secret123'); // Highly unstable locator
  await page.locator('.btn-submit-style-v2').click(); // Class-bound
  await page.waitForTimeout(5000); // Fatal Anti-pattern: Hardcoded wait

  await page.goto('https://my-app.internal');
  await page.locator('div > div > button').click(); // Layout bound
});
```

###  Good Practice (Enterprise-Grade, Deterministic, Abstracted)
```typescript
import { test, expect } from '@testing-library/or-native-framework';
import { LoginPage } from '../pages/LoginPage';
import { CartPage } from '../pages/CartPage';

test.describe('E2E Checkout Pipeline', () => {
  // Use state injection or hooks for hermetic isolation
  test.beforeEach(async ({ browserContext }) => {
    await injectPreAuthenticatedSession(browserContext, 'standard_customer');
  });

  test('should successfully complete purchase with a valid promo code', async ({ page }) => {
    const cartPage = new CartPage(page);

    // 1. Arrange & Navigate
    await cartPage.navigate();
    await cartPage.ensureCartIsLoaded(); // Explicit state hook

    // 2. Act (High-level business-focused abstractions)
    await cartPage.applyPromoCode('SUMMER26');
    await cartPage.proceedToCheckout();

    // 3. Assert (Web-first, deterministic auto-waiting assertion)
    await expect(cartPage.successNotification).toBeVisible();
    await expect(cartPage.orderTotal).toHaveText('\$85.00');
  });
});
```

---

## 🔍 Context Anchors for Debugging & Refactoring
When assigned to fix a broken or flaky test:
1. **Scan for Waits**: Locate any execution-pausing calls and replace them with event-driven conditional waits or reactive assertions.
2. **Audit Assertions**: Ensure assertions verify the *state change* resulting from an action, rather than assuming a static timeline.
3. **Isolate Failures**: Check if the test fails when run completely standalone 20 times in a row (`--repeat-each=20`). If it passes standalone but fails in a suite, hunt for leaky states or shared variables.