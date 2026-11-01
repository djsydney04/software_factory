# Software Factory

A quiet place to steer persistent software-building agents.

One main agent holds a project's goal, delegates bounded work, verifies results and brings back reviewable changes. A Rust service owns persistent state; terminal and desktop clients share its conversation and progress views.

Status: architecture and visual design, not yet a runnable application. The repository was empty at the start of this project. Claude through the Cursor CLI produced the current redesign: Conversation, Progress and Reviews are separate destinations, and the latest Operating Desk pass makes Progress a compact operations dashboard with a smaller Sundial radar. The visual direction is awaiting selection before implementation.

- [Getting started and implementation milestones](docs/getting-started.md)
- [Architecture and shared contract](docs/architecture.md)
- [Harness, recovery and experiments](docs/harness.md)
- [Provider protocols and verified local probes](docs/providers.md)
- [Release acceptance](docs/acceptance.md)
- [Research and sources](docs/research.md)
- [Revised visual direction](docs/design/README.md)

The intended stack is Rust, SQLite, Axum, Ratatui and Tauri. Codex app-server is the first execution integration. Other providers and deployment connections must report observed capability rather than simulated activity.

## Current design proposal

[Conversation](docs/design/operating-desk/conversation.png) · [Progress / Sundial radar](docs/design/operating-desk/progress.png) · [Reviews](docs/design/operating-desk/reviews.png)

![Software Factory conversation design proposal](docs/design/operating-desk/conversation.png)
