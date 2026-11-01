# Software Factory

A quiet place to steer persistent software-building agents.

One main agent holds a project's goal, delegates bounded work, verifies results and brings back reviewable changes. A Rust service owns persistent state; terminal and desktop clients share its conversation and progress views.

Status: architecture and visual design, not yet a runnable application. The repository was empty at the start of this project. Claude through the Cursor CLI produced the current redesign: Conversation, Progress and Reviews are separate destinations, and the latest Charcoal Desk pass adds a workspace overview, project-scoped tabs and explicit model pickers in a compact dark operations dashboard. The visual direction is awaiting selection before implementation.

- [Getting started and implementation milestones](docs/getting-started.md)
- [Architecture and shared contract](docs/architecture.md)
- [Harness, recovery and experiments](docs/harness.md)
- [Provider protocols and verified local probes](docs/providers.md)
- [Release acceptance](docs/acceptance.md)
- [Research and sources](docs/research.md)
- [Revised visual direction](docs/design/README.md)

The intended stack is Rust, SQLite, Axum, Ratatui and Tauri. Codex app-server is the first execution integration. Other providers and deployment connections must report observed capability rather than simulated activity.

## Current design proposal

[Workspace overview](docs/design/charcoal-desk/overview.png) · [Conversation](docs/design/charcoal-desk/conversation.png) · [Progress / Sundial radar](docs/design/charcoal-desk/progress.png) · [Reviews](docs/design/charcoal-desk/reviews.png)

![Software Factory workspace design proposal](docs/design/charcoal-desk/overview.png)
