---
name: functional-expert
description: Trigger this skill automatically when writing tests to validate business requirements, API contracts, features, or UI components from an end-user perspective, without managing full cross-system E2E infrastructure.
formats: [".spec.ts", ".test.js", "test_functional.py", "_functional_test.go", "ControllerTest.java", "Component.test.tsx"]
---

## 🎯 Activation Intent
Activate this skill to write functional and component tests. Your goal is to verify that a specific slice of the application (an API endpoint, a complex UI form, or a business feature) behaves exactly as specified by the product requirements. Focus heavily on input-output contracts, permissions, validation messages, and user-visible outcomes.

---

## ⛔ Absolute Anti-Patterns (Zero Tolerance)
1. **Testing internal states over visible behavior**:
   - Never assert against private object states, component internal variables, or local React/Vue states.
   - *Reason*: Functional testing focuses on *what* the feature does, not *how* it does it. If the UI framework or state structure changes but the feature works, this test should pass.
2. **Missing negative validation scenarios**:
   - Never write a functional test suite that only covers the positive "happy path" without testing error feedback.
   - *Reason*: Missing business validations, bad inputs, and error UI feedback are the most common points of functional breakage.
3. **Deep database mocking inside API controllers**:
   - Never create complex mock layers that return perfect domain models for every possible internal method call in an API controller test. Instead, mock the boundary or verify the integrated endpoint contract directly.

---

## ✅ Core Architectural Guidelines

### 1. Requirements-Driven Scenarios
* **Feature Correspondence**: Every test block (`describe`/`it`) must explicitly map to a business requirement, user story, or acceptance criteria (e.g., "should reject checkout if cart is empty").
* **Data-Driven Feature Coverage**: Use parameterized tests (e.g., `test.each`) when a single business rule applies across multiple varying user types or roles.

### 2. UI Component Isolation (Testing Library Rules)
* **User Actions Over Events**: Prefer firing realistic user actions (e.g., `@testing-library/user-event`) over raw programmatic event dispatching (`fireEvent.click`).
* **Accessible Queries**: Rely on accessible queries (`getByRole`, `getByLabelText`) to find UI elements. This ensures your functional test naturally validates basic digital accessibility alongside feature correctness.

### 3. API Contract and Status Enforcement
* **Payload Compliance**: Validate that responses match the defined OpenAPI schema or Zod/Joi validation contracts (correct keys, types, structure).
* **HTTP Semantics**: Enforce explicit status codes mapping to business outcomes: `422 Unprocessable Entity` or `400 Bad Request` for business logic violations, and `403 Forbidden` for missing roles.

---

## 📝 Concrete Reference Implementations

### ❌ Bad Practice (Testing private state, raw events, ignoring accessibility)
```typescript
// Refuse to generate brittle functional tests like this:
test('should toggle dropdown feature', async () => {
  const wrapper = shallowMount(MyDropdownComponent);
  
  // Anti-pattern: Mutating/asserting on private reactive state directly
  expect(wrapper.vm.isOpen).toBe(false); 
  
  // Anti-pattern: Firing a raw programmatic event instead of user simulation
  await wrapper.find('.raw-arrow-class').trigger('click'); 
  
  expect(wrapper.vm.isOpen).toBe(true);
});
```

###  Good Practice (User interaction simulation, semantic query lookup)
```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PromoCodeForm } from './PromoCodeForm';

describe('PromoCodeForm Functional Behavior', () => {
  it('should display an actionable error message when an invalid coupon is submitted', async () => {
    // 1. Arrange (Render in isolation)
    const user = userEvent.setup();
    const mockOnApply = jest.fn();
    render(<PromoCodeForm onApply={mockOnApply} />);

    // 2. Act (Simulate accessible user workflow)
    const inputField = screen.getByRole('textbox', { name: /promo code/i });
    const submitButton = screen.getByRole('button', { name: /apply/i });

    await user.type(inputField, 'INVALID_CODE_2026');
    await user.click(submitButton);

    // 3. Assert (Validate user-visible functional output)
    const errorMessage = await screen.findByText(/this promo code does not exist/i);
    expect(errorMessage).toBeInTheDocument();
    expect(mockOnApply).not.toHaveBeenCalled(); // Ensure the callback state was blocked
  });
});
```

---

## 🔍 Context Anchors for Code Generation & Reviews
When writing functional validations:
1. **The User Perspective Check**: Look at your selectors and actions. If a blind user or automated assistant cannot find or interact with that item via semantic hooks, rewrite the UI element or selector.
2. **The "So What?" Rule**: Ask if an assertion failure indicates a broken business requirement. If a failure only indicates that an internal variable name changed, remove the assertion.
