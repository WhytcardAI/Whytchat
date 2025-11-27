# WhytChat V1 Documentation

Welcome to the official technical documentation for **WhytChat V1**. This local-first AI chat application emphasizes privacy, performance, and an intuitive user experience.

## 📚 Documentation Structure

- **[Architecture](./ARCHITECTURE.md)**: High-level system design, tech stack (Tauri/Rust + React/Vite), and data models.
- **[Features](./FEATURES.md)**: Comprehensive list of current capabilities and planned roadmap items.
- **[RAG System](./RAG_SYSTEM.md)**: Detailed explanation of the Retrieval-Augmented Generation implementation, including the Global Library system.
- **[Business Plan](./BUSINESS_PLAN.md)**: Strategic vision, market analysis, and roadmap for WhytChat.

## 🚀 Quick Start

### Prerequisites

- **Rust**: v1.80.0 or higher
- **Node.js**: v18+
- **Tauri CLI**: v2.0.0-rc

### Installation

1.  Clone the repository.
2.  Install frontend dependencies:
    ```bash
    cd apps/desktop-ui
    npm install
    ```
3.  Run the application in development mode:
    ```bash
    npm run tauri dev
    ```

## 🏗️ Core Concepts

- **Local-First**: All data (chats, documents, vectors) stays on the user's machine.
- **Global Library**: A centralized repository for all documents, allowing files to be reused across multiple chat sessions.
- **Split View (Planned)**: A dual-pane interface for viewing documents alongside the chat stream.
- **Privacy**: No cloud dependencies for core functionality.