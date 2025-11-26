# WhytChat

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-000000.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0+-24C8DB.svg?style=flat&logo=tauri)](https://tauri.app/)

A local-first AI chat application built with Rust and React. WhytChat runs entirely on your machine - your data never leaves your computer.

## Features

- **100% Local**: AI models run on your machine. No cloud, no data sent anywhere.
- **RAG System**: Import your documents (PDF, Word, text) and chat with them.
- **Privacy First**: Your files stay on your computer.
- **Open Source**: MIT licensed, fully auditable code.

## Requirements

- Windows 10/11, macOS, or Linux
- 8GB RAM minimum (16GB recommended)
- 10GB disk space

## Quick Start

```bash
# Clone the repository
git clone https://github.com/WhytcardAI/WhytChat.git
cd WhytChat

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

On first launch, the application will download the required AI model (~5GB).

## Project Structure

```
WhytChat/
├── apps/
│   ├── core/           # Rust backend (Tauri)
│   │   └── src/        # Actors, database, RAG logic
│   └── desktop-ui/     # React frontend
│       └── src/        # Components, hooks, locales
├── Doc/                # Technical documentation
└── package.json
```

## Architecture

WhytChat uses a backend-centric architecture with an actor system:

```
React UI  <-->  Tauri IPC  <-->  Rust Actors
                                    |
                    +---------------+---------------+
                    |               |               |
                Supervisor      RAG Actor       LLM Actor
                    |               |               |
                SQLite          LanceDB        llama.cpp
```

- **Supervisor**: Orchestrates all actors and handles requests
- **RAG Actor**: Manages document ingestion and semantic search
- **LLM Actor**: Handles AI model inference via llama.cpp

## Technology Stack

| Component | Technology                           |
| --------- | ------------------------------------ |
| Backend   | Rust, Tauri 2.0                      |
| Frontend  | React, TypeScript, Tailwind CSS      |
| Database  | SQLite (sessions), LanceDB (vectors) |
| AI        | llama.cpp, FastEmbed                 |
| Build     | Vite, Cargo                          |

## Development

```bash
# Frontend lint
npm run lint

# Rust check
cd apps/core && cargo check

# Build release
npm run tauri build
```

## Environment Variables

Create a `.env` file from `.env.example`:

```bash
LLAMA_AUTH_TOKEN=your_secure_token
ENCRYPTION_KEY=32_character_encryption_key
TAVILY_API_KEY=optional_for_web_search
```

## Documentation

Technical documentation is available in the `Doc/` folder:

- [Architecture](Doc/ARCHITECTURE.md) - System design and components
- [RAG System](Doc/RAG_SYSTEM.md) - Document processing pipeline
- [User Manual](Doc/USER_MANUAL.md) - How to use the application

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m 'Add my feature'`
4. Push to your fork: `git push origin feature/my-feature`
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contact

- GitHub: [WhytcardAI](https://github.com/WhytcardAI)
- Email: jerome@whytcard.ai
