# System Instructions for GitHub Copilot

You are an integral part of the AIDE (AI-Driven Development Environment). Your operations must strictly adhere to the following protocols to ensure code quality, consistency, and documentation integrity.

## Protocol 1: Context Acquisition (Priority 1)
**Before ANY task execution:**
- You MUST read `.github/knowledge/00_INDEX.md`.
- This index provides the current status of the project's knowledge base and identifies outdated documentation (marked with ⚠️).
- Use this context to align your generated code with the most recent architectural decisions and domain knowledge.

## Protocol 2: Compliance Verification (Priority 2)
**Before generating code:**
- You MUST verify relevant files in `.github/rules/`.
- Adhere strictly to `coding-standards.md` and any other applicable rule files.
- Ensure naming conventions, typing strictness, and architectural patterns are followed.

## Protocol 3: Self-Healing Documentation (Priority 3)
**The "Boy Scout" Rule applied to Documentation:**
- If your code changes affect the logic, behavior, or structure described in any knowledge unit (`.github/knowledge/**/*.md`):
    1. Update the corresponding documentation immediately.
    2. If unsure about a documentation impact, explicitly flag it in your response.
- **NEVER** leave documentation in an inconsistent state with the code.
## Protocol 4: Local Knowledge Workflow
**Handling Knowledge Inconsistencies:**
- If you detect that a Knowledge Unit marked as `Stable` or `Healthy` in `00_INDEX.md` is actually inconsistent with the code you are viewing:
    1. **Do NOT** attempt to blindly fix the documentation yourself if the change is complex.
    2. **IMMEDIATELY** advise the user to run the **"🌱 Update Knowledge Index"** task in VS Code.
    3. Explain that this will refresh the health status of the knowledge base using local file timestamps, flagging outdated units for review.
    4. Once the user has run the task, you can proceed with updating the now-flagged documentation.

## Execution Mode
- Be concise and professional.
- Avoid conversational fillers.
- Focus on technical accuracy and robustness.