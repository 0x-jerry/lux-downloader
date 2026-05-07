---
name: developer
description: General-purpose software development with SOLID, DRY, and code quality principles. Use when writing, reviewing, or refactoring code.
---

# Developer Skill

General-purpose software development skill. Apply these principles when writing or reviewing any code.

## SOLID Principles

### Single Responsibility (SRP)
A module, class, or function should have exactly one reason to change — it should do one thing and do it well.
- If a function name needs "and" to describe it, split it.
- If a class has methods that change for different actors/reasons, split it.

### Open/Closed (OCP)
Code should be open for extension but closed for modification.
- Prefer composition and plugin-style patterns over modifying existing code to add behavior.
- Use interfaces/abstract classes to define contracts, allowing new implementations without touching callers.

### Liskov Substitution (LSP)
Subtypes must be substitutable for their base types without altering correctness.
- A subclass should not strengthen preconditions or weaken postconditions.
- If overriding a method, the override must honor the contract of the base method — don't throw unexpected exceptions or ignore required invariants.

### Interface Segregation (ISP)
Clients should not depend on methods they don't use. Keep interfaces small and focused.
- Split fat interfaces into smaller, role-specific ones.
- A consumer that only reads shouldn't know about write methods.

### Dependency Inversion (DIP)
Depend on abstractions, not concretions. High-level modules should not depend on low-level modules; both should depend on abstractions.
- Inject dependencies rather than instantiating them internally.
- Use dependency injection, factory patterns, or parameter passing to provide implementations.

## DRY (Don't Repeat Yourself)

Every piece of knowledge must have a single, unambiguous, authoritative representation in the system.
- **When to DRY**: Same logic/concept appears in 3+ places, or the duplication represents the same business rule/knowledge.
- **When NOT to DRY**: Two pieces of code look similar but serve different concerns or change for different reasons — extracting them couples unrelated things.
- **Signs of violation**: Copy-pasted blocks, magic values repeated across files, the same validation logic in multiple endpoints.

## Code Quality

- **Naming**: Names should reveal intent. A name that requires a comment to explain what it does is a bad name. Prefer `fetchUserById` over `get` — length is cheaper than ambiguity.
- **Functions**: Keep functions short and focused. A function should do one thing — its name says what, its body says how. If the body has sections separated by blank lines, extract them.
- **Avoid premature abstraction**: Don't abstract until the pattern is clear. Duplication is cheaper than the wrong abstraction — wait until you see the shape repeat before extracting.
- **No dead code**: Remove unused imports, variables, functions, and commented-out code. Version control keeps history.
- **Error handling**: Fail fast and loud. Handle errors at boundaries; don't swallow exceptions silently. Never catch an error just to log and re-throw — let it propagate.

## When Writing Code

1. Start with the simplest thing that works. Then refactor for clarity.
2. Write code for the reader, not the writer — code is read far more than it is written.
3. Every line of code should earn its keep. If you can't explain why it exists, delete it.
4. Respect existing conventions. Consistency within a codebase beats external best practices.


## Function Parameters Design

1. Prefer struct when parameters more than three.

## Other Tips

1. !!! Prefer refactor code instead of fair of breaking changes.
