# 🧠 Knowledge Base Index

_Last updated: 2025-11-26_

## 📚 Documentation Structure

| Document         | Location                                      | Description                                     |
| :--------------- | :-------------------------------------------- | :---------------------------------------------- |
| **Architecture** | `Doc/ARCHITECTURE.md`                         | System architecture, diagrams, tech stack       |
| **RAG System**   | `Doc/RAG_SYSTEM.md`                           | RAG implementation, text extraction, embeddings |
| **IA Internals** | `Doc/IA_INTERNALS.md`                         | Brain module, LLM, embeddings                   |
| **User Manual**  | `Doc/USER_MANUAL.md`                          | End-user guide                                  |
| **Processes**    | `Doc/PROCESSUS.md`                            | Business workflows, sequence diagrams           |
| **Standards**    | `Doc/STANDARDS.md`                            | Coding standards, best practices                |
| **Methodology**  | `Doc/METHODOLOGIE_DEV.md`                     | SDLC, Git workflow, testing                     |
| **Audit**        | `.github/Project_knowledge/AUDIT_CRITIQUE.md` | Technical debt, risks                           |

## 🎯 Quick Reference

### File Upload Flow

- **Entry Point**: `KnowledgeView` only (Import Data button)
- **Supported Formats**: `.txt`, `.md`, `.csv`, `.json`, `.pdf`, `.docx`, `.doc`
- **Text Extraction**: `apps/core/src/text_extract.rs`
- **Max Size**: 10 MB per file

### Key Commands

- `upload_file_for_session`: Upload + extract + ingest
- `link_library_file_to_session`: Link existing file (no re-ingestion)
- `get_session_files`: Get files linked to session
- `reindex_library`: Re-process all library files

## 🛡️ Legend

- 🟢 Stable: Validated and reliable knowledge.
- 🟡 Draft: Work in progress.
- ⚠️ Outdated: Linked code has changed since last verification. Needs review.
- ✅ Healthy: Up to date with codebase.
