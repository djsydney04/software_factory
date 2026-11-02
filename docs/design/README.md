# Software Factory visual direction

Status: current Claude design proposal, November 1, 2026; awaiting visual selection. These are illustrative mockups, not a runnable factory or evidence of real PRs, checks or deployments.

The latest user request is authoritative: more compact controls, clearer hierarchy, no changes embedded in Radar or other monitoring views, and a distinctive original radar design. The user specifically requested Claude through the Cursor CLI. Claude inspected the previous screens and references, authored the redesign, and refined the radar in the same session.

[Open the current Soft Charcoal redesign and screens](soft-charcoal/README.md).

The latest pass uses softer warm charcoal surfaces, quieter dividers, stone buttons, no emoji, text navigation and explicit provider/model/effort pickers. Run links open project-scoped run details and attempt history. A workspace Overview operates across projects; each project owns Conversation, Progress and Reviews. [Charcoal Desk](charcoal-desk/README.md), [Operating Desk](operating-desk/README.md) and [the preceding Claude mockups](claude-redesign/README.md) remain available for comparison.

## Current hierarchy

- **Workspace Overview:** project goals, states, pending reviews and scoped controls across projects.
- **Project Conversation:** one main agent and one steering input. Details unfold on selection. Results link to Reviews without embedding the review workspace.
- **Project Progress:** Graph, Board and Radar show work and ownership. Radar contains only Plan, Build, Check and Waiting, with no changes, PRs, previews, diffs or merge controls.
- **Run detail within Progress:** inspect a specific attempt, tool activity, changes, checks and artifacts; steer or interrupt active work and inspect previous attempts.
- **Project Reviews:** changes have their own destination, with a compact queue and a large selected preview/diff, checks, limitations and actions.
- **Settings:** a secondary bottom utility.

The sidebar lists Overview and projects. Each selected project has a scoped goal header and Conversation / Progress / Reviews tabs. The target at 1440×900 uses a 176px sidebar, 40px workspace header and readable 14–16px main text. Exact raster dimensions are illustrative; implementation must verify actual geometry, accessibility and reflow.

The smaller Sundial radar uses faint charcoal rings and restrained delegation lines. The raster is an illustrative stage overview; exact ages stay in the task table. Angle shows current stage; radius shows time in stage. Only active work and retained blocked inputs appear. Reviews and completed changes stay outside the radar. The TUI shares this hierarchy and data, using terminal-friendly task lists and age indicators.

## Source and history

- [Current Claude brief](soft-charcoal/claude-design-brief.md) and [execution provenance](soft-charcoal/provenance.json).

- [Claude's original brief](claude-redesign/claude-design-brief.md), with the initial Radar proposal superseded by [its refinement](claude-redesign/radar-refinement.md).
- [Execution provenance](claude-redesign/provenance.json) and [render prompts](claude-redesign/render-prompts.md).
- [Earlier notes](pre-claude-notes.md), superseded by the current direction. The earlier `compact-*.png` images put reviews inside monitoring views and must not be used as the build target.

Design: Claude Opus via Cursor CLI. Raster rendering: built-in Image Gen. The latest screens use a fictional Tasklet project; the Green Room preview belongs to the earlier design set.
