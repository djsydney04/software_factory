I'll read the operating-desk README and look at its three images before revising.I read the operating-desk README and inspected all three PNGs directly.

# Brief: Software Factory, "Charcoal Desk"

## What's wrong with the operating-desk images

- **Pictograms are still everywhere:** coloured state shapes, nav icons, the boxed Lead glyph, context icons and tag chips.
- **Copy is too long.** Conversation added a "thread context" panel nobody asked for.
- **The Sundial's numeric rings misplace ages.** For example, "9m" sits outside the 15m ring.
- **Sidebar widths drift** between screens.
- **No scope above a single project.** There is nowhere to operate across projects.

## What changed

1. **Charcoal theme.** Neutral charcoal surfaces, warm white text and muted accents used only as text colour on state labels. Navigation and states are plain text. The only icons are monochrome chevron, close and send controls.
2. **Two-level structure.** The sidebar has Overview, then a Projects list. Opening a project shows a 64px project header with its name, goal, state, a pause control, and tabs for Conversation, Progress and Reviews. Every chat and review sits inside a project.
3. **Overview:**
   - A cross-project table: goal and focus, state, active agents, pending reviews, default worker model, and Pause/Open controls.
   - A short "Needs you" list where every item is named by project and opens that project.
   - The workspace default model applies only to new projects.
4. **Model pickers show provider, model and effort.** Each picker states its scope:
   - The lead picker applies from the next turn.
   - A task override applies to its next run.
   - Running work never switches silently.
   - Codex 5.3 and Claude Opus 5.5 are example names only; availability is checked when a run starts.
5. **Sundial** is a 220px illustrative stage overview with no numeric rings. Exact ages stay in the table, and reviews and done work are excluded.

## Tokens

- **Surfaces:** background #1B1A19, sidebar #161514, top bar #1F1E1C, panel #242321, menu or active row #2B2A27.
- **Hairlines:** #34322F (standard), #423F3B (strong).
- **Text:** #EDE8DF (primary), #B5AEA3 (secondary), #8A847A (tertiary).
- **State text colours:** Running #8FA88A, Needs you #C9A36A, Blocked #C27B66, Paused #8A847A.
- **Diff backgrounds:** added #22301F, removed #3A2622.
- **Type:** Inter 14/20 body, 13px secondary, 11px uppercase labels. JetBrains Mono 12px for commits and durations.
- **Geometry:** sidebar 176px, top bar 40px, 28px rows and controls, 34px table rows.

## Sample data

Sample data is the same on every screen:

| Project | Goal | Focus | State | Active agents | Pending reviews | Default worker model |
| --- | --- | --- | --- | --- | --- | --- |
| Tasklet | Ship offline-first task lists | Local SQLite persistence | Running | 4 | 2 | Codex 5.3 · High |
| Software Factory | Recover runs across restarts | Run lease recovery | Running | 2 | 1 | Claude Opus 5.5 · High |
| Docs Kit | Publish a searchable docs site | none | Paused | 0 | 0 | Codex 5.3 · Medium |

## Overview

```text
Polished flat 1440x900 macOS desktop app screenshot, one screen, no device frame, no collage. Dark charcoal theme. No emoji; no icons except monochrome functional chevrons, close and send; no avatars, logos, pictograms or colored status shapes. SHELL (identical on all screens): background #1B1A19. Top bar 40px, #1F1E1C, 1px bottom hairline #34322F: traffic lights left; at x92 breadcrumb 13px #B5AEA3 "Example workspace / Overview"; right 12px #8A847A "Local daemon · connected". Sidebar 176px, #161514, right hairline #34322F, plain text 13px rows 28px: "Overview"; label "PROJECTS" 11px uppercase #8A847A; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused" right-aligned. Active row #2B2A27 fill, 6px radius, #EDE8DF text; inactive #B5AEA3. Bottom: "Settings". Type Inter: body 14/20 #EDE8DF, secondary 13px #B5AEA3, labels 11px uppercase tracking 0.06em #8A847A; JetBrains Mono 12px for commits, durations. Panels #242321, menus #2B2A27, radius 6px controls 8px panels, no gradients. State is plain text: "Running" #8FA88A, "Needs you" #C9A36A, "Blocked" #C27B66, "Paused" #8A847A. Primary button #EDE8DF fill, #1B1A19 text; secondary buttons 1px #423F3B outline. Fictional example data.

SCREEN: Overview active in sidebar. Content x176 to 1440, 32px padding.
Header 56px: "Overview" 18px semibold; right secondary button "New project".

Projects table in #242321 panel, 8px radius, header 11px uppercase: Project, Goal and focus, State, Active agents, Reviews, Default worker model, blank actions. Rows 52px, hairline separators:
- "Tasklet" 14px semibold | "Ship offline-first task lists" 14px / "Focus: local SQLite persistence" 13px secondary | "Running" | "4" | "2 pending" | "Codex 5.3 · High" with chevron | buttons "Pause", "Open".
- "Software Factory" | "Recover runs across restarts" / "Focus: run lease recovery" | "Running" | "2" | "1 pending" | "Claude Opus 5.5 · High" | "Pause", "Open".
- "Docs Kit" | "Publish a searchable docs site" / "No current focus" | "Paused" | "0" | "None" | "Codex 5.3 · Medium" | "Resume", "Open".
Below table, 12px tertiary: "Default model applies to new runs in that project. Running work is unchanged."

Section "Needs you" 11px label, panel with two 40px rows:
- "Tasklet" tertiary, "Migrate fixture data: write access outside worktree requested", "Needs you" amber text, mono "1h 05m", secondary button "Open in Tasklet".
- "Tasklet" tertiary, "Push reminder scheduling: run failed, timeout", "Blocked" rust text, mono "2h 20m", "Open in Tasklet".

Section "Workspace" 11px label, one 40px row: "Default model for new projects" 14px, picker "Codex 5.3 · High" with chevron, tertiary 12px "Example models; availability checked at run start".

Large calm empty space below. No charts, no tiles, no scores.
```

## Conversation

```text
Polished flat 1440x900 macOS desktop app screenshot, one screen, no device frame, no collage. Dark charcoal theme. No emoji; no icons except monochrome functional chevrons, close and send; no avatars, logos, pictograms or colored status shapes. SHELL (identical on all screens): background #1B1A19. Top bar 40px, #1F1E1C, 1px bottom hairline #34322F: traffic lights left; at x92 breadcrumb 13px #B5AEA3 "Example workspace / Tasklet"; right 12px #8A847A "Local daemon · connected". Sidebar 176px, #161514, right hairline #34322F, plain text 13px rows 28px: "Overview"; label "PROJECTS" 11px uppercase #8A847A; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused" right-aligned. Active row #2B2A27 fill, 6px radius, #EDE8DF text; inactive #B5AEA3. Bottom: "Settings". Type Inter: body 14/20 #EDE8DF, secondary 13px #B5AEA3, labels 11px uppercase tracking 0.06em #8A847A; JetBrains Mono 12px for commits, durations. Panels #242321, menus #2B2A27, radius 6px controls 8px panels, no gradients. State is plain text: "Running" #8FA88A, "Needs you" #C9A36A, "Blocked" #C27B66, "Paused" #8A847A. Primary button #EDE8DF fill, #1B1A19 text; secondary buttons 1px #423F3B outline. Fictional example data.
PROJECT HEADER: below top bar, 64px, bottom hairline. Line 1: "Tasklet" 16px semibold, goal "Ship offline-first task lists" 14px #B5AEA3; right "Running" text and secondary 28px "Pause project". Line 2: text tabs 13px "Conversation", "Progress", "Reviews 2"; active tab #EDE8DF with 2px underline, others #8A847A.

SCREEN: Tasklet active in sidebar; Conversation tab active. Log column 760px, left-aligned, 40px from sidebar. Right rail 280px, #1F1E1C, left hairline.

Log, 20px between entries, names 12px semibold, mono tertiary times:
- "You · 1h": "This week tasks must survive offline. Keep sync out of scope."
- "Lead · 56m": "Saved as the goal. Four tasks, three delegated." Inline hairline table, 30px rows: "Persist tasks in SQLite · Build · Codex 5.3"; "Keyboard shortcuts for lists · Check · Claude Opus 5.5"; "Validate offline data · Plan · Lead".
- "Lead · 28m": "Due-date sorting is ready. 5 checks passed on 9c41e07." 36px row: "Add due-date sorting #18", right "Open in Reviews" with chevron.
- "Lead · 12m": "Fixture migration needs write access outside its worktree. Paused until you decide." Buttons "Allow once" primary, "Skip task" secondary.

Composer 72px, #242321, 1px #423F3B border: placeholder "Steer the lead agent"; bottom-left pickers "Queue" and "Lead: Claude Opus 5.5 · High" with chevrons; right 28px warm-white circular send arrow.
OPEN MENU rising from the model picker, 300px, #2B2A27, subtle shadow: label "LEAD MODEL"; rows "Codex 5.3" with tertiary "Codex", "Claude Opus 5.5" with tertiary "Claude" and monochrome checkmark; label "EFFORT"; segmented "Low | Medium | High", High selected; footer 12px tertiary "Applies from the next turn. The current turn is unchanged. Example models."

Right rail: label "DELEGATED"; 40px rows, title 13px, state text right: "Persist tasks in SQLite" Running; "Local search index" Running; "Keyboard shortcuts for lists" Running; "Migrate fixture data" Needs you; "Push reminder scheduling" Blocked. Link "Open Progress". Short copy, generous spacing.
```

## Progress

```text
Polished flat 1440x900 macOS desktop app screenshot, one screen, no device frame, no collage. Dark charcoal theme. No emoji; no icons except monochrome functional chevrons, close and send; no avatars, logos, pictograms or colored status shapes. SHELL (identical on all screens): background #1B1A19. Top bar 40px, #1F1E1C, 1px bottom hairline #34322F: traffic lights left; at x92 breadcrumb 13px #B5AEA3 "Example workspace / Tasklet"; right 12px #8A847A "Local daemon · connected". Sidebar 176px, #161514, right hairline #34322F, plain text 13px rows 28px: "Overview"; label "PROJECTS" 11px uppercase #8A847A; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused" right-aligned. Active row #2B2A27 fill, 6px radius, #EDE8DF text; inactive #B5AEA3. Bottom: "Settings". Type Inter: body 14/20 #EDE8DF, secondary 13px #B5AEA3, labels 11px uppercase tracking 0.06em #8A847A; JetBrains Mono 12px for commits, durations. Panels #242321, menus #2B2A27, radius 6px controls 8px panels, no gradients. State is plain text: "Running" #8FA88A, "Needs you" #C9A36A, "Blocked" #C27B66, "Paused" #8A847A. Primary button #EDE8DF fill, #1B1A19 text; secondary buttons 1px #423F3B outline. Fictional example data.
PROJECT HEADER: below top bar, 64px, bottom hairline. Line 1: "Tasklet" 16px semibold, goal "Ship offline-first task lists" 14px #B5AEA3; right "Running" text and secondary 28px "Pause project". Line 2: text tabs 13px "Conversation", "Progress", "Reviews 2"; active tab #EDE8DF with 2px underline, others #8A847A.

SCREEN: Tasklet active in sidebar; Progress tab active. Main x176 to 1080, 32px padding; right column 360px, #1F1E1C, left hairline.

Toolbar 40px: segmented "List | Board | Graph", List active; right 12px tertiary "Active work only".

"NEEDS YOU" label; two 40px rows in #242321 panel: "Migrate fixture data", "Write access outside worktree", "Needs you" amber, mono "1h 05m", "Allow once" primary, "Skip" secondary. "Push reminder scheduling", "Run failed: timeout", "Blocked" rust, mono "2h 20m", "Retry", "Steer" secondary.

"TASKS" label; table, 34px rows, header 11px uppercase: Task, Stage, State, Agent, In stage, Next run model.
- "Validate offline data" | Plan | Running | "Lead · Claude Opus 5.5 · High" | 9m | "Project default" tertiary with chevron
- "Persist tasks in SQLite" (selected, #2B2A27 row) | Build | Running | "Codex 5.3 · High" | 22m | "Project default"
- "Local search index" | Build | Running | "Codex 5.3 · Medium" | 48m | "Claude Opus 5.5 · High" with chevron, override set
- "Keyboard shortcuts for lists" | Check | Running | "Claude Opus 5.5 · Medium" | 6m | "Project default"
- "Migrate fixture data" | Waiting | Needs you | "Codex 5.3 · High" | 1h 05m | "Project default"
- "Push reminder scheduling" | Waiting | Blocked | "Codex 5.3 · High" | 2h 20m | "Project default"
Below table 12px tertiary: "Overrides apply to the next run. Running work keeps its model. Project default: Codex 5.3 · High."

Right column: label "STAGE OVERVIEW · ILLUSTRATIVE". Quiet 220px Sundial: two faint unlabeled concentric rings, thin crossing hairlines, 11px uppercase quadrant labels PLAN top-left, BUILD top-right, CHECK bottom-right, WAITING bottom-left. Centre 28px #2B2A27 disc "Lead" 11px. Small 6px warm-white neutral dots with 11px short labels, thin hairline arms to centre; waiting dots hollow with dotted arms. Search index farther out than Persist SQLite; Shortcuts near centre. Caption 12px tertiary: "Farther out means longer in stage. Exact ages are in the table."
Below: selected detail: "Persist tasks in SQLite" 15px semibold; mono "codex 5.3 · tasklet/sqlite-persist · run 2"; "Next run: project default"; buttons "Steer", "Interrupt". No reviews, PRs or diffs.
```

## Reviews

```text
Polished flat 1440x900 macOS desktop app screenshot, one screen, no device frame, no collage. Dark charcoal theme. No emoji; no icons except monochrome functional chevrons, close and send; no avatars, logos, pictograms or colored status shapes. SHELL (identical on all screens): background #1B1A19. Top bar 40px, #1F1E1C, 1px bottom hairline #34322F: traffic lights left; at x92 breadcrumb 13px #B5AEA3 "Example workspace / Tasklet"; right 12px #8A847A "Local daemon · connected". Sidebar 176px, #161514, right hairline #34322F, plain text 13px rows 28px: "Overview"; label "PROJECTS" 11px uppercase #8A847A; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused" right-aligned. Active row #2B2A27 fill, 6px radius, #EDE8DF text; inactive #B5AEA3. Bottom: "Settings". Type Inter: body 14/20 #EDE8DF, secondary 13px #B5AEA3, labels 11px uppercase tracking 0.06em #8A847A; JetBrains Mono 12px for commits, durations. Panels #242321, menus #2B2A27, radius 6px controls 8px panels, no gradients. State is plain text: "Running" #8FA88A, "Needs you" #C9A36A, "Blocked" #C27B66, "Paused" #8A847A. Primary button #EDE8DF fill, #1B1A19 text; secondary buttons 1px #423F3B outline. Fictional example data.
PROJECT HEADER: below top bar, 64px, bottom hairline. Line 1: "Tasklet" 16px semibold, goal "Ship offline-first task lists" 14px #B5AEA3; right "Running" text and secondary 28px "Pause project". Line 2: text tabs 13px "Conversation", "Progress", "Reviews 2"; active tab #EDE8DF with 2px underline, others #8A847A.

SCREEN: Tasklet active in sidebar; Reviews tab active. Queue x176 to 456, #1F1E1C, right hairline. Workspace x456 to 1440.

Queue: label "READY · 2". Rows 52px, two lines: "Add due-date sorting" 14px semibold / mono "#18 · Codex 5.3 · checks passed" (selected, #2B2A27 fill, 2px warm-white left bar); "Empty state for new lists" / "#17 · Claude Opus 5.5 · checks passed". Label "CHANGES REQUESTED · 1": "Rename list action" / "#15 · checks stale" in amber text. Collapsed "Merged · 3" with chevron.

Header 52px: "Add due-date sorting" 18px semibold, mono "#18", mono tertiary "tasklet/due-date-sort · 9c41e07"; right segmented "Preview | Diff | Split", Split active; text link "Open PR". One line 13px secondary: "Sorts lists by due date, undated tasks last."

Body split, 16px gutter. Left preview 420px, #242321 backdrop, 11px label "Example preview". Centered light phone screen 300px, #F6F3EE, rounded: "Groceries" 20px semibold, pill "Sort: Due date"; rows with hollow circles and right dates: "Buy oat milk · Today", "Return library books · Tomorrow", "Pick up prescription · Fri", "Call plumber · No date" muted; struck-through "Water plants".
Right diff pane, #1F1E1C, hairline, 8px radius: file bar mono "ListView.swift +24 −6", right "3 files" with chevron. ~26 lines Swift, JetBrains Mono 12px, tertiary line numbers: sortedTasks property putting nil due dates last; removed lines #3A2622, added lines #22301F; Menu("Sort") block added.

Evidence bar 56px, top hairline: label "CHECKS ON 9C41E07"; inline text "swift test passed", "lint passed", "ui snapshot passed", "build passed", "fixtures passed" 13px, separators; then amber text "1 limitation: iPad layouts not tested". Right: "Request changes" secondary, "Merge 9c41e07" primary. Compact, the change dominates.
```