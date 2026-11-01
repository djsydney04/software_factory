I inspected all three PNGs directly. Everything below is read-only design output.

# Brief: Software Factory, "Operating Desk" pass

## What's wrong with the current images

- **Conversation:** About 60% of the canvas is empty. A narrow message column floats in a wide field, and the composer is oversized. Nothing on screen shows what delegated work is doing while you steer. The lead's bar-chart glyph looks like an analytics icon.
- **Radar:** The instrument takes two-thirds of the screen. Tick rims, hatching and a caption add decoration but no information. Who is working and what needs you is squeezed into a thin side list. The centre says "idle" while the header says "Running".
- **Reviews:** This is the strongest screen, but the queue rows are airy and the goal header repeats without purpose. The preview also depends on a branded external app.
- **Shell:** A 184px sidebar, tall nav rows and a 44px header are loose for an operating tool. State is shown by colour alone.

## What changed

1. **Operating shell.** A single 40px top bar shows the project, an Example tag, the goal, the current focus and live state counts on every screen. The sidebar is 168px with 28px rows, and its footer shows whether the local daemon is connected.
2. **State uses shape as well as colour,** everywhere:
   - Running: moss filled circle.
   - Checking: slate half-filled circle.
   - Needs you: amber diamond.
   - Blocked: brick square.
3. **Conversation is a working log.** All messages are left-aligned in a 760px column. A pinned decision tray sits above a compact composer with a Queue/Now steering toggle. A 296px "Delegated from this thread" rail lists work. There is still only one conversation and no worker inboxes.
4. **Progress is an operations desk.** It has a "Needs you" strip with inline actions, a dense active-work table and a lead decision log. The Sundial shrinks to a 300px instrument in the right column, with details of the selected task below it.
5. **Reviews is denser.** The queue is 288px wide with 52px rows, and the header is one line. Preview and diff are split. An evidence bar shows checks tied to the tested commit, plus limitations and actions.
6. **Zero emoji.** The only icons are 16px monochrome line icons with a 1.5px stroke, used only where they label a function. No KPIs, percentages or scores. Counts appear only as plain factual tallies.

## Tokens

- **Colour:**
  - Canvas #F4F1EA, panel #FAF8F3, rail #F7F4EE, sidebar #EDE9E1, top bar #EFEBE3.
  - Hairline #DDD7CB.
  - Text: charcoal #23211E (primary), #6E675D (secondary), #8F887D (tertiary).
  - Primary button #1F1D1A.
  - States: moss #4F7A57, slate #5B6B7A, amber #B7791F (tint #FBF3E4), brick #A8442E (tint #F6E6E0).
  - Diff: added #E6EEDF, removed #F3E1DA.
- **Type:**
  - Inter: body 14/20, secondary 13/18, labels 11px uppercase with 0.06em tracking, titles 15–18px semibold.
  - JetBrains Mono 12px for IDs, durations, branches and code.
- **Geometry:**
  - 4px grid. Controls are 28px tall with 6px radius; panels have 8px radius.
  - Rows: 28px (nav), 34px (table), 52px (queue).
  - No shadows or gradients.

## What the radar means

- **Quadrant is the current stage:** Plan top-left, Build top-right, Check bottom-right, Waiting bottom-left (needs-you and blocked).
- **Distance from the centre is time in the current stage.** Rings mark 15m, 1h and 4h, so older work sits farther out.
- **Arms:** Running work has a tapered gnomon-shadow arm from the Lead disc. Waiting and blocked work has a static dotted arm, so it never implies a live process.
- **Exclusions:** Reviews, merged work and done work never appear on the radar.
- **Restraint:** No sweep, no tick rim, no scores. The selected table row and the radar mark highlight together.

## Sample content

All content is the fictional "Tasklet" project, marked Example: an offline-first task-list app.

Active work, which must stay consistent across all three screens (4 running, 1 needs you, 1 blocked):

- **Plan:** Sync conflict notice (Lead, 9m)
- **Build:** Persist tasks in SQLite (Codex, 22m) and Local search index (Codex, 48m)
- **Check:** Keyboard shortcuts for lists (Claude, 6m)
- **Needs you:** Migrate fixture data (1h 05m)
- **Blocked:** Push reminder scheduling (2h 20m)

Review #18, "Add due-date sorting", appears only in Conversation (as a link) and in Reviews.

## Conversation

```text
Polished flat 1440x900 macOS desktop app screenshot, single screen, no device frame, no collage, zero emoji, no illustrations. Warm paper palette. SHELL (identical on every screen): canvas #F4F1EA. Top bar 40px full width, #EFEBE3, 1px bottom hairline #DDD7CB. Left: macOS traffic lights at x16; at x92 project switcher "Tasklet" 13px semibold charcoal #23211E with small chevron-down line icon, then tiny outlined pill "Example" 11px #6E675D. Thin vertical divider, goal "Ship offline-first task lists" 13px medium charcoal, then "Focus: local SQLite persistence" 13px #6E675D. Right: 12px state counters: moss #4F7A57 filled dot "4 running", amber #B7791F small diamond "1 needs you", brick #A8442E small square "1 blocked"; then 28px outlined button with pause line icon and label "Pause". Left sidebar 168px below top bar, #EDE9E1, 1px right hairline. Nav rows 28px, 16px monochrome line icons (1.5px stroke, #6E675D), 13px labels: Conversation, Progress, Reviews with right-aligned mono count "2". Active row #FAF8F3 fill, 6px radius, charcoal label. Sidebar bottom: "Settings" row with gear icon; below it 11px #8F887D "Local daemon · connected" with tiny moss dot. Type Inter: body 14/20, secondary 13px, labels 11px uppercase tracking 0.06em #8F887D; JetBrains Mono 12px for IDs and durations. Hairlines #DDD7CB, panels #FAF8F3, radius 6px controls, 8px panels, no shadows, no gradients. All content is fictional example data.

SCREEN: Conversation (active nav row). Main area x168 to x1144: a left-aligned 760px log column starting 48px from sidebar. Right rail x1144 to 1440, #F7F4EE, 1px left hairline.

Log, top to bottom, 20px between entries, no avatars, no bubbles:
1. "You" 12px semibold, mono "1h" tertiary. Text with 2px charcoal left rule: "This week tasks must survive offline. Keep sync out of scope."
2. "Lead" with tiny 14px monochrome square-in-square glyph. "Saved as the goal. I split it into four tasks and delegated three." Below, an inline hairline-framed mini table, three 30px rows: state glyph, title 13px, stage, agent, mono age: moss dot "Persist tasks in SQLite · Build · Codex · 22m"; slate half-circle "Keyboard shortcuts for lists · Check · Claude · 6m"; moss dot "Sync conflict notice · Plan · Lead · 9m".
3. "Lead": "Due-date sorting is ready for review. 5 checks passed on 9c41e07. I couldn't test iPad layouts, so that is listed as a limitation." Below, one 36px hairline row: layers line icon, "Add due-date sorting", mono "#18", right-aligned "Open in Reviews" with arrow icon. No preview, no diff.
4. "Lead": "Migrating fixture data needs write access outside its worktree. I paused it until you decide."

Pinned above composer, full column width: decision tray 52px, #FBF3E4 fill, 1px #E8D3AE border, 8px radius: amber diamond, "Needs you" 11px uppercase, "Migrate fixture data: write access to fixtures/ for this run only" 14px; right: "Allow once" charcoal #1F1D1A filled 28px button, "Skip task" outlined.
Composer 64px, #FAF8F3, 1px hairline, 8px radius: placeholder "Steer the lead agent" 14px tertiary; bottom-left small 12px toggle "Queue" with chevron; right 28px charcoal circular send button with up-arrow icon.

Right rail: header "Delegated from this thread" 11px uppercase, top line "Lead · planning next step" 12px with moss dot. Four 44px two-line rows, hairline separators: moss dot "Persist tasks in SQLite" / mono "codex · build · 22m"; slate half-circle "Keyboard shortcuts for lists" / "claude · check · 6m"; amber diamond "Migrate fixture data" / "needs you · 1h 05m"; brick square "Push reminder scheduling" / "interrupted · decision needed · 2h 20m". Bottom text link "Open Progress" with arrow icon. Generous but disciplined whitespace, calm, legible.
```

## Progress

```text
Polished flat 1440x900 macOS desktop app screenshot, single screen, no device frame, no collage, zero emoji, no illustrations. Warm paper palette. SHELL (identical on every screen): canvas #F4F1EA. Top bar 40px full width, #EFEBE3, 1px bottom hairline #DDD7CB. Left: macOS traffic lights at x16; at x92 project switcher "Tasklet" 13px semibold charcoal #23211E with small chevron-down line icon, then tiny outlined pill "Example" 11px #6E675D. Thin vertical divider, goal "Ship offline-first task lists" 13px medium charcoal, then "Focus: local SQLite persistence" 13px #6E675D. Right: 12px state counters: moss #4F7A57 filled dot "4 running", amber #B7791F small diamond "1 needs you", brick #A8442E small square "1 blocked"; then 28px outlined button with pause line icon and label "Pause". Left sidebar 168px below top bar, #EDE9E1, 1px right hairline. Nav rows 28px, 16px monochrome line icons (1.5px stroke, #6E675D), 13px labels: Conversation, Progress, Reviews with right-aligned mono count "2". Active row #FAF8F3 fill, 6px radius, charcoal label. Sidebar bottom: "Settings" row with gear icon; below it 11px #8F887D "Local daemon · connected" with tiny moss dot. Type Inter: body 14/20, secondary 13px, labels 11px uppercase tracking 0.06em #8F887D; JetBrains Mono 12px for IDs and durations. Hairlines #DDD7CB, panels #FAF8F3, radius 6px controls, 8px panels, no shadows, no gradients. All content is fictional example data.

SCREEN: Progress operations desk (active nav row). Main x168 to x1072 with 24px padding; right column x1072 to 1440, #F7F4EE, left hairline.

Main subheader 44px: "Progress" 15px semibold; segmented control 26px "List | Board | Graph", List active; right 12px tertiary "Reviews and done work excluded".

"Needs you" label, then two 44px rows, 8px radius: (a) #FBF3E4 fill, amber diamond, "Migrate fixture data" 14px semibold, "Write access outside worktree requested" 13px secondary, mono "1h 05m", buttons "Allow once" charcoal filled, "Skip" outlined, 28px. (b) #F6E6E0 fill, brick square, "Push reminder scheduling", "Run failed: timeout. Retry or reassign", mono "2h 20m", buttons "Retry", "Steer" outlined.

"Active work" table, 34px rows, hairline separators, headers 11px uppercase: State, Task, Stage, Agent, In stage, Last event. Rows: moss dot "Sync conflict notice" Plan, Lead, 9m, "drafting acceptance criteria"; moss dot "Persist tasks in SQLite" Build, Codex, 22m, "wrote migration 003" (selected: #FAF8F3 fill, 2px charcoal left bar); moss dot "Local search index" Build, Codex, 48m, "indexing fixtures"; slate half-circle "Keyboard shortcuts for lists" Check, Claude, 6m, "running cargo test". Last-event text mono 12px.

"Lead log" below: four 24px mono lines, tertiary timestamps "4m", "12m", "31m", "1h": "delegated shortcuts check to claude", "paused fixture migration pending approval", "moved due-date sorting to review #18", "split goal into four tasks".

Right column: label "Sundial · stage and time in stage". A 300px circular instrument, engraved-paper feel: three thin concentric rings labeled mono 11px "15m", "1h", "4h", increasing outward; crossing hairlines make quadrants; inside the rim 11px uppercase PLAN top-left, BUILD top-right, CHECK bottom-right, WAITING bottom-left. Centre 36px charcoal disc "Lead" in white 11px. Running marks: 8px moss dots with soft tapered warm-gray gnomon-shadow arms from centre. Plan: "Sync conflict" inside 15m ring. Build: "Persist SQLite" just beyond 15m ring with 14px charcoal selection ring; "Search index" near 1h ring. Check: slate half-circle "Shortcuts" close to centre. Waiting: amber diamond "Fixture data" just outside 1h ring; brick square "Push reminders" between 1h and 4h; both with thin static dotted arms. Mark labels 11px. No ticks, no sweep.
Legend line 11px under it: "Farther out = longer in current stage".
Selected detail: "Persist tasks in SQLite" 14px semibold; mono "codex · tasklet/sqlite-persist · run 2"; "Build for 22m"; 13px acceptance "Tasks and lists survive app restart offline"; buttons "Steer", "Interrupt" outlined 28px. Dense, calm, operational.
```

## Reviews

```text
Polished flat 1440x900 macOS desktop app screenshot, single screen, no device frame, no collage, zero emoji, no illustrations. Warm paper palette. SHELL (identical on every screen): canvas #F4F1EA. Top bar 40px full width, #EFEBE3, 1px bottom hairline #DDD7CB. Left: macOS traffic lights at x16; at x92 project switcher "Tasklet" 13px semibold charcoal #23211E with small chevron-down line icon, then tiny outlined pill "Example" 11px #6E675D. Thin vertical divider, goal "Ship offline-first task lists" 13px medium charcoal, then "Focus: local SQLite persistence" 13px #6E675D. Right: 12px state counters: moss #4F7A57 filled dot "4 running", amber #B7791F small diamond "1 needs you", brick #A8442E small square "1 blocked"; then 28px outlined button with pause line icon and label "Pause". Left sidebar 168px below top bar, #EDE9E1, 1px right hairline. Nav rows 28px, 16px monochrome line icons (1.5px stroke, #6E675D), 13px labels: Conversation, Progress, Reviews with right-aligned mono count "2". Active row #FAF8F3 fill, 6px radius, charcoal label. Sidebar bottom: "Settings" row with gear icon; below it 11px #8F887D "Local daemon · connected" with tiny moss dot. Type Inter: body 14/20, secondary 13px, labels 11px uppercase tracking 0.06em #8F887D; JetBrains Mono 12px for IDs and durations. Hairlines #DDD7CB, panels #FAF8F3, radius 6px controls, 8px panels, no shadows, no gradients. All content is fictional example data.

SCREEN: Reviews (active nav row). Queue x168 to x456, #F7F4EE, right hairline. Review workspace x456 to 1440.

Queue: label "Ready · 2". Rows 52px, two lines, hairline separators: "Add due-date sorting" 14px semibold / mono 12px "#18 · codex · 5/5 checks · 14m" with small moss check line icon (selected: #FAF8F3 fill, 2px charcoal left bar); "Empty state for new lists" / "#17 · claude · 4/4 checks · 40m". Label "Changes requested · 1": "Rename list action" / "#15 · codex · updated, checks stale" with small amber diamond. Collapsed row "Merged · 3" with chevron-right icon.

Workspace header 52px: "Add due-date sorting" 18px semibold, mono "#18", mono tertiary "tasklet/due-date-sort · 9c41e07"; right: segmented control 26px "Preview | Diff | Split" (Split active), text link "Open PR" with external-link icon. Below, one line 13px secondary: "Sorts each list by due date with undated tasks last. Adds a sort menu to the list header."

Body split, 16px gutter. Left preview pane 400px wide, #EAE6DD backdrop, 8px radius; small corner label 11px "Example preview · simulator". Centered flat phone screen 300px wide, white #FFFDF9, rounded: header "Groceries" 20px semibold, pill "Sort: Due date" with chevron; task rows with hollow circle checkboxes and right-aligned dates 13px: "Buy oat milk · Today" (date in brick), "Return library books · Tomorrow", "Pick up prescription · Fri", "Call plumber · No date" muted gray; one completed row "Water plants" struck through at bottom. Simple, neutral, no logos.
Right diff pane fills remaining width, #FAF8F3, 1px hairline, 8px radius: file bar "ListView.swift  +24 −6" mono, right "3 files" with chevron. Code JetBrains Mono 12px with tertiary line numbers, ~28 lines of Swift: a sortedTasks computed property sorting by dueDate with nil last; removed lines on #F3E1DA with "−", added lines on #E6EEDF with "+"; a Menu("Sort") block added.

Evidence bar 48px across workspace bottom, top hairline: moss check icon "5 checks passed on 9c41e07" 13px, text link "Details"; thin divider; amber diamond "1 limitation: iPad layouts not tested" link; right: "Request changes" outlined 30px, "Merge" charcoal #1F1D1A filled 30px with white label. Compact, clear, the change dominates.
```