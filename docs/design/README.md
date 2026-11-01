# Software Factory visual direction

Status: current Claude design proposal, November 1, 2026; awaiting visual selection. These are illustrative mockups, not a runnable factory or evidence of real PRs, checks or deployments.

The latest user request is authoritative: more compact controls, clearer hierarchy, no changes embedded in Radar or other monitoring views, and a distinctive original radar design. The user specifically requested Claude through the Cursor CLI. Claude inspected the previous screens and references, authored the redesign, and refined the radar in the same session.

[Open the current Operating Desk redesign and screens](operating-desk/README.md).

The latest pass removes emoji, reduces chrome and puts active-work rows and blocker actions first. A smaller Sundial accompanies task details. [The preceding Claude mockups](claude-redesign/README.md) remain available for comparison.

## Current hierarchy

- **Conversation:** one main agent and one steering input. Details unfold on selection. Results link to Reviews without embedding the review workspace.
- **Progress:** Graph, Board and Radar show work and ownership. Radar contains only Plan, Build, Check and Waiting, with no changes, PRs, previews, diffs or merge controls.
- **Reviews:** changes have their own destination, with a compact queue and a large selected preview/diff, checks, limitations and actions.
- **Settings:** a secondary bottom utility.

A project switcher replaces the large wordmark and repeated project list. The goal, focus and state share one compact header. The target at 1440×900 uses a 168px sidebar, 40px header and readable 14–16px main text. Exact raster dimensions are illustrative; implementation must verify actual geometry, accessibility and reflow.

The Sundial radar uses engraved paper-like rings and tapering delegation shadows. Angle shows current stage; radius shows time in stage. Only active work and retained blocked inputs appear. Reviews and completed changes stay outside the radar. The TUI shares this hierarchy and data, using terminal-friendly task lists and age indicators.

## Source and history

- [Claude's original brief](claude-redesign/claude-design-brief.md), with the initial Radar proposal superseded by [its refinement](claude-redesign/radar-refinement.md).
- [Execution provenance](claude-redesign/provenance.json) and [render prompts](claude-redesign/render-prompts.md).
- [Earlier notes](pre-claude-notes.md), superseded by the current direction. The earlier `compact-*.png` images put reviews inside monitoring views and must not be used as the build target.

Design: Claude Opus via Cursor CLI. Raster rendering: built-in Image Gen. The Green Room example preview follows the maintained brand guide and uses an existing labeled example Collection screenshot as reference.
