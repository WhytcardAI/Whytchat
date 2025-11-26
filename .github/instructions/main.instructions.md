---
applyTo: "**"
---

# Project Knowledge Management Workflow

This project uses a dynamic knowledge base located in `.github/Project_knowledge`. All agents must actively contribute to and utilize this knowledge base to ensure continuous learning and context preservation.

## 1. Knowledge Retrieval (First Step)

Before starting any complex task, research, or implementation:

1.  **Check Existing Knowledge**: Search the `.github/Project_knowledge` directory for relevant documentation.
2.  **Read**: If a relevant file exists, read it to understand established patterns, decisions, and library usages.

## 2. Knowledge Creation Trigger

You must create or update a knowledge file when:

- You research a new library or technology (e.g., using `context7` or `tavily`).
- You make a significant architectural decision.
- You solve a complex, non-obvious bug (post-mortem).
- You discover important project-specific constraints or patterns.

## 3. Automated Research & Documentation Process

When you need to acquire new knowledge, follow this strict workflow:

### Phase 1: Planning

- Use **`mcp_sequential-th_sequentialthinking`** to break down what you need to learn.
- Identify key questions: "How do we use X in this project?", "What are the best practices for Y?", "What is the solution to error Z?".

### Phase 2: Gathering Information

- **For Libraries/Docs**: Use **`mcp_context7_resolve-library-id`** and **`mcp_context7_get-library-docs`** to fetch official documentation.
- **For General Info/Troubleshooting**: Use **`mcp_tavily-mcp_tavily-search`** to find articles, discussions, or solutions.
- **For Deep Dives**: Use **`mcp_tavily-mcp_tavily-extract`** to read the full content of promising URLs found.

### Phase 3: Synthesis & Storage

- **Synthesize** the gathered information into a clear, concise Markdown format.
- **Contextualize** it for _this specific project_ (e.g., "Here is how we use React Query with our custom fetch wrapper").
- **Save** the file in `.github/Project_knowledge/` with a descriptive name (e.g., `react-query-patterns.md`, `auth-architecture.md`).

## 4. Knowledge File Template

When creating a new file in `.github/Project_knowledge`, use this structure:

```markdown
# [Topic Name]

## Context

Why is this relevant to our project? (e.g., "Used for state management", "Solution to issue #123")

## Key Concepts

- Concept A: Explanation...
- Concept B: Explanation...

## Project-Specific Implementation

How do we implement this _here_?
(Provide code snippets that match our project's style and patterns)

## References

- [Link to official docs]
- [Link to relevant issue/discussion]
```

## 5. Continuous Improvement

- If you find a knowledge file is outdated or incomplete during your work, **update it** immediately.
- Do not rely on stale memory; trust the `.github/Project_knowledge` as the source of truth.
