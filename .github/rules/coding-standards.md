# Coding Standards

## General Principles
- **Clean Code**: Code must be readable, maintainable, and self-documenting.
- **DRY (Don't Repeat Yourself)**: Avoid duplication. Extract common logic into reusable functions or components.
- **KISS (Keep It Simple, Stupid)**: Prefer simple solutions over complex ones.

## TypeScript / JavaScript
- **Strict Typing**: Use TypeScript strict mode. Avoid `any`.
- **Naming Conventions**:
    - `camelCase` for variables and functions.
    - `PascalCase` for classes, interfaces, and types.
    - `UPPER_CASE` for constants.
- **Async/Await**: Prefer `async/await` over raw Promises (`.then()`).

## Comments
- **Why, not What**: Comments should explain *why* something is done, not *what* the code does (unless it's complex regex or algorithms).
- **JSDoc**: Use JSDoc for public APIs and complex functions.