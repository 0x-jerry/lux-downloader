---
name: js-developer
description: JavaScript/TypeScript developer. Use when writing, reviewing, or refactoring JS/TS code.
---

> Read `developer` skill first — all principles there apply to JS/TS code too.

## TypeScript

Always use TypeScript in strict mode. Never write plain JavaScript.

### `import type`

Always use `import type` for types imported only for type-checking. This is enforced by `verbatimModuleSyntax` and ensures types are erased at compile time.

```ts
import type { User, Config } from './types'
import { fetchUser } from './api'
```

## Module System

Use ESM exclusively (`"type": "module"` in package.json).

- Use `import`/`export` syntax. Never `require()`.
- Use barrel exports (`index.ts` re-exporting from sibling modules) for clean public APIs.
- No default exports — named exports are greppable and refactor-safe.

## Function Design

### Named vs Arrow

- Use **named function declarations** for exported functions and top-level helpers. They hoist, have clearer stack traces, and read more naturally:

```ts
export function fetchUser(id: string): Promise<User> { ... }
function sanitize(input: string): string { ... }
```

- Use **arrow functions** for inline callbacks and anonymous functions:

```ts
const ids = users.map((u) => u.id)
button.addEventListener('click', () => handleClick())
```

### Async/Await

Always use `async/await` over raw `.then()` chains (Except top-level script). It's easier to read, debug, and reason about.

```ts
const data = await fetch(url)
const parsed = await data.json()
```

For concurrent operations, use `Promise.all` or `Promise.allSettled`:

```ts
const [user, posts] = await Promise.all([fetchUser(id), fetchPosts(id)])
```

### Parameter Design

- Prefer objects (destructured parameters) when a function takes more than 3 arguments.
- Use default parameter values instead of manual `??` checks in the body.

## Naming Conventions

- **Files**: kebab-case (`fetch-user.ts`, `use-tasks.ts`)
- **Functions/Variables**: camelCase (`fetchUser`, `isLoading`)
- **Types/Interfaces**: PascalCase (`User`, `ApiResponse`)
- **Constants**: UPPER_SNAKE_CASE for true constants (`MAX_RETRIES`, `DEFAULT_TIMEOUT`)
- **Boolean variables**: `is`, `has`, `should` prefix (`isValid`, `hasError`, `shouldRetry`)
- **Prefers single quote** and omit semi.
- **Composables/hooks**: `use` prefix (`useConfig`, `useCache`)

## When Writing JS/TS Code

1. Start with the types. Define the interfaces first — they shape the implementation.
3. Avoid `any`. Use `unknown` when the type is truly unknown, and narrow it with type guards before use.
4. Prefer template literals over string concatenation.
5. Use `for...of` for iteration that needs `await` inside; use `.map`/`.filter`/`.reduce` for data transformation.
6. Use optional chaining (`?.`) and nullish coalescing (`??`) to write concise null-safe code.

## Runtime Check

1. Always check the closest package.json and lock file(bun.lock/pnpm-lock.json/package-lock.json) before run any script, then choose the right runtime binary(bun/pnpm/node)
2. Always check the content of the closest package.json before run any script.