I read the charcoal-desk README and inspected all four PNGs: overview, conversation, progress and reviews.

# Brief: Software Factory, "Soft Charcoal"

## What still feels hard in the charcoal-desk images

- The background renders close to black.
- Buttons are stark white and look heavy.
- Full-height rules and boxed tables make the grid rigid.
- The Overview wordmark is inconsistent with the other screens.
- The model menu crowds the message log.
- The Progress detail says "run 2" but you can't open it.
- Diff colours are saturated, and the checks strip is cramped.

## Changes in this pass

1. **Lifted warm charcoal.** Sidebar and window chrome share #1E1D1B. Content sits on a #232220 sheet with a 10px top-left radius. Layers are separated by surface shifts instead of borders, and there are no full-height rules.
2. **Softer type and controls.**
   - Text is #E6E1D8. The primary button is soft stone #CFC8BC instead of white, and secondary buttons are tonal #2E2C29.
   - Controls have 7px radius and panels 10px.
   - Table rows are 38px, separators are faint (#312F2C), and diff tints are low-saturation.
3. **Run drilldown.** Progress gets a Run column with underlined links such as "Run 02". Clicking one opens a run detail inside Progress, scoped to the project.
   - It has a Back to Progress link and the path Tasklet / Progress / Persist tasks in SQLite / Run 02.
   - It shows the run's fixed provider, model and effort, a separate Next run picker, and an attempt selector.
   - Tabs: Activity, Changes, Checks, Artifacts.
   - Steer and Interrupt are available while the run is active.
   - Retry appears only on the failed Run 01, and it is disabled while Run 02 is active.
   - Merge stays in Reviews.
4. **Honest checks.** The fixtures check shows "Running" and lint shows "Not run". The activity log shows tool calls and output only, never reasoning.
5. **Sundial** stays a quiet 200px illustrative stage view. Exact ages stay in the table, and reviews are excluded.

## Data (unchanged, all Example)

- **Tasklet:** 4 running, 2 pending reviews.
- **Tasks:** Validate offline data, Persist tasks in SQLite (Run 02), Local search index, Keyboard shortcuts for lists, Migrate fixture data, Push reminder scheduling.
- **Example models:** Codex 5.3 and Claude Opus 5.5.

## Overview

```text
Polished flat 1440x900 macOS app screenshot, one screen, no collage. Soft warm charcoal dark theme. No emoji, icons, avatars, logos or status dots; only monochrome chevrons and send arrow. SHELL: chrome and sidebar #1E1D1B, muted gray window controls. Top bar 40px, no divider: breadcrumb 13px #ADA79D at x92; right 12px "Local daemon · connected". Sidebar 176px, text-only 13px, 30px rows: "Overview"; label "PROJECTS"; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused"; bottom "Settings". Active row #2E2C29, 7px radius. Content sheet #232220, 10px top-left radius, no hard borders. Inter 14/20 #E6E1D8, secondary 13px #ADA79D, labels 11px uppercase #8E887E; JetBrains Mono 12px for IDs, commits, times. Panels #292825, menus #33312E soft shadow, faint separators #312F2C, 7px controls, 10px panels, no gradients. State text: Running #9DB394, Needs you #D2AE78, Blocked/Failed #CF8B76, Paused #8E887E. Primary button #CFC8BC, dark text; secondary #2E2C29. Example data.

SCREEN: Overview active; breadcrumb "Example workspace / Overview". Sheet padding 32px. "Overview" 18px semibold; right secondary "New project".

Projects panel, 10px radius, header labels: Project, Goal, State, Agents, Reviews, Default worker model. Rows 52px, faint separators, rows clickable:
- "Tasklet" semibold | "Ship offline-first task lists" / secondary "Focus: local SQLite persistence" | Running | 4 | 2 pending | "Codex 5.3 · High" chevron | text button "Pause", secondary "Open"
- "Software Factory" | "Recover runs across restarts" / "Focus: run lease recovery" | Running | 2 | 1 pending | "Claude Opus 5.5 · High" | "Pause", "Open"
- "Docs Kit" | "Publish a searchable docs site" / "No current focus" | Paused | 0 | None | "Codex 5.3 · Medium" | "Resume", "Open"
Caption 12px tertiary: "Defaults apply to new runs. Active runs keep their model."

Label "NEEDS YOU"; panel, two 44px rows: "Tasklet · Migrate fixture data" / "Write access outside worktree" | Needs you | mono "1h 05m" | link "Open run". "Tasklet · Push reminder scheduling" / "Run 01 failed: timeout" | Blocked | "2h 20m" | "Open run".

Label "WORKSPACE"; one 44px row: "Default model for new projects", picker "Codex 5.3 · High" chevron, tertiary "Example models".

Calm space below. No charts, tiles or scores.
```

## Conversation

```text
Polished flat 1440x900 macOS app screenshot, one screen, no collage. Soft warm charcoal dark theme. No emoji, icons, avatars, logos or status dots; only monochrome chevrons and send arrow. SHELL: chrome and sidebar #1E1D1B, muted gray window controls. Top bar 40px, no divider: breadcrumb 13px #ADA79D at x92; right 12px "Local daemon · connected". Sidebar 176px, text-only 13px, 30px rows: "Overview"; label "PROJECTS"; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused"; bottom "Settings". Active row #2E2C29, 7px radius. Content sheet #232220, 10px top-left radius, no hard borders. Inter 14/20 #E6E1D8, secondary 13px #ADA79D, labels 11px uppercase #8E887E; JetBrains Mono 12px for IDs, commits, times. Panels #292825, menus #33312E soft shadow, faint separators #312F2C, 7px controls, 10px panels, no gradients. State text: Running #9DB394, Needs you #D2AE78, Blocked/Failed #CF8B76, Paused #8E887E. Primary button #CFC8BC, dark text; secondary #2E2C29. Example data.
PROJECT HEADER: "Tasklet" 16px semibold, goal "Ship offline-first task lists" #ADA79D; right "Running", secondary "Pause project". Text tabs "Conversation", "Progress", "Reviews 2"; active has 2px soft underline.

SCREEN: Tasklet active; breadcrumb "Example workspace / Tasklet"; Conversation tab. Log column 720px; right rail 272px inset panel #262522.

Log, 24px gaps, names 12px semibold, mono times:
- "You · 1h": "This week tasks must survive offline. Keep sync out of scope."
- "Lead · 56m": "Saved as the goal. Four tasks, three delegated." Inset list, 32px rows: "Persist tasks in SQLite · Build · Codex 5.3"; "Keyboard shortcuts for lists · Check · Claude Opus 5.5"; "Validate offline data · Plan · Lead".
- "Lead · 28m": "Due-date sorting is ready for review." Row "Add due-date sorting #18", link "Open in Reviews".
- "Lead · 12m": "Fixture migration needs write access outside its worktree." Buttons "Allow once" primary, "Skip task" secondary.

Composer 76px #292825, 10px radius: placeholder "Steer the lead agent"; bottom-left pickers "Queue" and "Lead · Claude Opus 5.5 · High" with chevrons; right 30px soft circular send arrow.
Open menu rising from model picker, 288px #33312E: label "LEAD MODEL"; rows "Codex 5.3" tertiary "Codex", "Claude Opus 5.5" tertiary "Claude" with "Current"; label "EFFORT"; segmented "Low | Medium | High", High active; footer 12px "Applies from the next turn. Example models." Menu does not cover message text.

Right rail "DELEGATED", 36px rows, state text right: Persist tasks in SQLite Running; Local search index Running; Keyboard shortcuts for lists Running; Migrate fixture data Needs you; Push reminder scheduling Blocked. Link "Open Progress".
```

## Progress

```text
Polished flat 1440x900 macOS app screenshot, one screen, no collage. Soft warm charcoal dark theme. No emoji, icons, avatars, logos or status dots; only monochrome chevrons and send arrow. SHELL: chrome and sidebar #1E1D1B, muted gray window controls. Top bar 40px, no divider: breadcrumb 13px #ADA79D at x92; right 12px "Local daemon · connected". Sidebar 176px, text-only 13px, 30px rows: "Overview"; label "PROJECTS"; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused"; bottom "Settings". Active row #2E2C29, 7px radius. Content sheet #232220, 10px top-left radius, no hard borders. Inter 14/20 #E6E1D8, secondary 13px #ADA79D, labels 11px uppercase #8E887E; JetBrains Mono 12px for IDs, commits, times. Panels #292825, menus #33312E soft shadow, faint separators #312F2C, 7px controls, 10px panels, no gradients. State text: Running #9DB394, Needs you #D2AE78, Blocked/Failed #CF8B76, Paused #8E887E. Primary button #CFC8BC, dark text; secondary #2E2C29. Example data.
PROJECT HEADER: "Tasklet" 16px semibold, goal "Ship offline-first task lists" #ADA79D; right "Running", secondary "Pause project". Text tabs "Conversation", "Progress", "Reviews 2"; active has 2px soft underline.

SCREEN: Tasklet active; breadcrumb "Example workspace / Tasklet"; Progress tab. Main 32px padding; right column 336px.
Toolbar: segmented "List | Board | Graph", List active; right filter "Active work" chevron.

"NEEDS YOU" panel, 44px rows: "Migrate fixture data" / "Write access outside worktree" | Needs you | 1h 05m | "Allow once" primary, "Skip". "Push reminder scheduling" / "Run 01 failed: timeout" | Blocked | 2h 20m | "Retry", "Steer".

"TASKS" panel table, 38px rows, faint separators, headers: Task, Run, Stage, State, Agent, In stage, Next run. Run column: underlined mono links; rows end in chevron.
- Validate offline data | Run 01 | Plan | Running | Lead · Claude Opus 5.5 · High | 9m | Default
- Persist tasks in SQLite | Run 02 | Build | Running | Codex 5.3 · High | 22m | Default — hovered row #2E2C29, pointer on "Run 02"
- Local search index | Run 01 | Build | Running | Codex 5.3 · Medium | 48m | Claude Opus 5.5 · High chevron
- Keyboard shortcuts for lists | Run 01 | Check | Running | Claude Opus 5.5 · Medium | 6m | Default
- Migrate fixture data | Run 01 | Waiting | Needs you | Codex 5.3 · High | 1h 05m | Default
- Push reminder scheduling | Run 01 | Waiting | Blocked | Codex 5.3 · High | 2h 20m | Default
Caption 12px tertiary: "Next run overrides never change an active run."

Right: label "STAGE OVERVIEW · ILLUSTRATIVE"; quiet 200px Sundial: two faint rings, soft cross lines, labels PLAN, BUILD, CHECK, WAITING; centre disc "Lead"; small off-white marks with 11px labels, thin muted arms; waiting marks hollow with dotted arms. Caption "Exact ages in table."
Below, panel: "Persist tasks in SQLite" / mono "Run 02 · Codex 5.3 · High" / link "Open Run 02"; secondary "Steer", "Interrupt". No reviews, PRs or diffs.
```

## Reviews

```text
Polished flat 1440x900 macOS app screenshot, one screen, no collage. Soft warm charcoal dark theme. No emoji, icons, avatars, logos or status dots; only monochrome chevrons and send arrow. SHELL: chrome and sidebar #1E1D1B, muted gray window controls. Top bar 40px, no divider: breadcrumb 13px #ADA79D at x92; right 12px "Local daemon · connected". Sidebar 176px, text-only 13px, 30px rows: "Overview"; label "PROJECTS"; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused"; bottom "Settings". Active row #2E2C29, 7px radius. Content sheet #232220, 10px top-left radius, no hard borders. Inter 14/20 #E6E1D8, secondary 13px #ADA79D, labels 11px uppercase #8E887E; JetBrains Mono 12px for IDs, commits, times. Panels #292825, menus #33312E soft shadow, faint separators #312F2C, 7px controls, 10px panels, no gradients. State text: Running #9DB394, Needs you #D2AE78, Blocked/Failed #CF8B76, Paused #8E887E. Primary button #CFC8BC, dark text; secondary #2E2C29. Example data.
PROJECT HEADER: "Tasklet" 16px semibold, goal "Ship offline-first task lists" #ADA79D; right "Running", secondary "Pause project". Text tabs "Conversation", "Progress", "Reviews 2"; active has 2px soft underline.

SCREEN: Tasklet active; breadcrumb "Example workspace / Tasklet"; Reviews tab. Queue 272px inset panel #262522; workspace fills rest.

Queue "READY · 2", 52px rows: "Add due-date sorting" / mono "#18 · Codex 5.3 · checks passed", selected #2E2C29; "Empty state for new lists" / "#17 · Claude Opus 5.5 · checks passed". "CHANGES REQUESTED · 1": "Rename list action" / amber "#15 · checks stale". "Merged · 3" chevron.

Header: "Add due-date sorting" 18px semibold, mono "#18 · tasklet/due-date-sort · 9c41e07"; right segmented "Preview | Diff | Split", Split active; link "Open PR". Secondary line "Sorts lists by due date, undated last."

Split body, 16px gutter, 10px panels. Left preview 400px #292825, label "Example preview": soft light phone screen #EEEAE3, 300px: "Groceries", pill "Sort: Due date", rows with hollow circles and right dates: Buy oat milk Today; Return library books Tomorrow; Pick up prescription Fri; Call plumber No date muted; struck-through Water plants.
Right diff #262522: file bar "ListView.swift +24 −6", "3 files" chevron; ~24 lines Swift, mono 12px, muted line numbers; sortedTasks puts nil dates last; Menu("Sort") added; removed rows #3A2A26, added rows #283126, low saturation.

Footer panel 52px: label "CHECKS ON 9C41E07"; "5 passed" then quiet text "swift test · lint · ui snapshot · build · fixtures"; amber "1 limitation: iPad not tested". Right: "Request changes" secondary, "Merge" primary.
```

## Run detail

```text
Polished flat 1440x900 macOS app screenshot, one screen, no collage. Soft warm charcoal dark theme. No emoji, icons, avatars, logos or status dots; only monochrome chevrons and send arrow. SHELL: chrome and sidebar #1E1D1B, muted gray window controls. Top bar 40px, no divider: breadcrumb 13px #ADA79D at x92; right 12px "Local daemon · connected". Sidebar 176px, text-only 13px, 30px rows: "Overview"; label "PROJECTS"; "Tasklet", "Software Factory", "Docs Kit" with tertiary "Paused"; bottom "Settings". Active row #2E2C29, 7px radius. Content sheet #232220, 10px top-left radius, no hard borders. Inter 14/20 #E6E1D8, secondary 13px #ADA79D, labels 11px uppercase #8E887E; JetBrains Mono 12px for IDs, commits, times. Panels #292825, menus #33312E soft shadow, faint separators #312F2C, 7px controls, 10px panels, no gradients. State text: Running #9DB394, Needs you #D2AE78, Blocked/Failed #CF8B76, Paused #8E887E. Primary button #CFC8BC, dark text; secondary #2E2C29. Example data.
PROJECT HEADER: "Tasklet" 16px semibold, goal "Ship offline-first task lists" #ADA79D; right "Running", secondary "Pause project". Text tabs "Conversation", "Progress", "Reviews 2"; active has 2px soft underline.

SCREEN: Tasklet active; breadcrumb "Example workspace / Tasklet"; Progress tab. Sub-line 13px: link "‹ Back to Progress", tertiary "Tasklet / Progress / Persist tasks in SQLite / Run 02".

Header: "Persist tasks in SQLite" 18px semibold, "Running"; attempt selector "Run 02 · Running" chevron, OPEN menu 300px #33312E: "Run 02 · Running · 22m" with "Current"; "Run 01 · Failed · timeout" with disabled tertiary "Retry"; footer 12px "Retry unavailable while Run 02 is active." Right: "Steer" primary, "Interrupt" secondary.

Facts row, label/value, no boxes: Model "Codex 5.3 · High" tertiary "fixed for this run"; Next run picker "Project default"; Goal "Tasks survive restart offline"; Branch mono "tasklet/sqlite-persist"; Base "main 3f2a91c"; Head "b7e04d2".

Tabs: "Activity" active, "Changes 4", "Checks", "Artifacts".
Activity panel, mono 12px rows, tertiary elapsed times:
00:00 started worktree from 3f2a91c
03:12 edit Sources/Store/SQLiteStore.swift
06:05 add migrations/003_create_tasks.sql
09:30 swift build · exit 0
14:48 swift test StoreTests · exit 1 · 2 failed
15:20 edit SQLiteStore.swift
18:02 swift test StoreTests · exit 0
21:40 scripts/check-fixtures.sh · running
Live tail beneath, dimmer: "loaded fixtures/lists.json: 12 lists", "simulating restart", "restoring tasks from SQLite…", ending "Running", no result.

Right column 288px: "CHECKS ON B7E04D2": swift build Passed; StoreTests Passed; fixtures Running; lint Not run. Tertiary "Merge happens in Reviews."
```