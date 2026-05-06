---
name: vue-developer
description: Vue 3 Composition API developer. Use when writing, reviewing, or refactoring Vue components, composables.
---

> Read `developer` skill first — all principles there apply to Vue code too.

# Vue Developer Skill

Vue 3 + Composition API + TypeScript. Focus on correctness, readability, and leveraging Vue's reactivity system properly.

## Composition API Only

Always use `<script setup lang="ts">`. Never use Options API.

- Use `ref` for primitives and simple values; `reactive` for objects when you want deep reactivity on all properties.
- Prefer `reactive` by default — it's more clear for reader.
- Always define XXXProps and XXXEmits interfaces if needed.
- Use `defineModel` for two-way binding proxies instead of manual emit handlers.

## Global State Design

A global state should be used by multiple place, only make global state if it used by different place.

Always prefer local state, not global state.

The priority of state design(lower is higher):

1. local state
2. parent state
3. global state

## Component Design

There are two type of components, first is pure component, the other is business component.

### Pure Component

This kind of component is not care about business logic, it should design as it should be for common use or specific use case, like UI library.

### Business Component

This kind of component should be tied with user's demand, and it should be designed as a self-consistent component.
It should only expose necessary props and emits.

## When Writing Vue Code

1. Template first — sketch what the UI should look like before wiring logic.
2. Props are the API of your component. Design them so the caller can't misuse them by accident.
3. If something is hard to explain or looks hacky, it's wrong. Vue's design pushes you toward idiomatic solutions — trust it.
4. TypeScript generics for component refs: `const el = ref<InstanceType<typeof MyComponent>>()`.
