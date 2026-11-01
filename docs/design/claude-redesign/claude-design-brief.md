# Software Factory redesign: compact hierarchy, separate Reviews and the Sundial radar

I inspected all five images: the three current mocks, the factory.ai reference and the Codex reference. Mock PNGs are 1024×640 renders, so I read their proportions, not pixels.

## 1. Design diagnosis

- **All three screens look the same.** All three mocks end in the same review pane: a "Melo" preview, Preview/Diff tabs, Request changes and Merge change. Switching between Conversation, Graph and Radar only changes the top 140px. The navigation suggests three places, but the page shows one.
- **The Radar is a stepper, not a radar.** It is six circles in a row (Signal, Plan, Build, Verify, Review, Ship) with single-digit counts. The Signal and Ship rings are borrowed from factory.ai, and they carry "Watching" and "Healthy", which nothing has observed. It doesn't show who works on what, how long anything has waited, or what needs the human.
- **Status is repeated, but the goal never appears.** The status of the same work appears up to four times: "Green Room · Working" in the top bar, "2 agents working · 1 ready" in the overview strip, "Tests passed · a4f3c2e" next to the title, and "1 ready for review" in Conversation. Meanwhile "Discovery improvements" is a category, not the goal. The sidebar also spends about 90px on a "Factory" wordmark plus a project list that a switcher could replace.
- **The hierarchy is inverted.** The largest and darkest text on every screen is the sample app's "Find your next favorite" (about 24px bold). Product chrome sits at 13px in light grey with flat weights, so the preview outranks the product. In Conversation, a roughly 240px chat column is mostly empty and the composer floats.
- **The preview isn't Green Room.** It shows a generic streaming app called "Melo", but Green Room is a native concert journal with Records/Grid views. "Review" is just a big screenshot: no diff beside it, no check detail, no limitations, and no sign of which commit was tested.

## 2. Hierarchy and rules

```
Shell: sidebar (project switcher + destinations)  /  top bar (goal · current focus · factory state)
├── Conversation   talk to the one lead agent             (no child views)
├── Progress       watch the work                          Graph | Board | Radar  → selected task detail
├── Reviews        decide on changes                       change list → selected change: Split | Preview | Diff → checks, limitations
└── Settings       utility; pinned to the bottom of the sidebar, not a peer destination
```

The TUI uses the same tree: keys 1/2/3 switch destinations, the header line holds the goal and focus, and Settings is a command.

1. **Each destination has one job.** Conversation is for talking, Progress for watching, Reviews for deciding. A screen never borrows another screen's body.
2. **Changes live only in Reviews.** Other screens may reference a change with one line of text ("Open in Reviews"). They never show a preview, diff, change list or merge control.
3. **The goal and current focus live only in the top bar.** Clicking it discloses the full saved goal. They never reappear as a workspace heading.
4. **Hierarchy comes from position, weight and one level of surface, not from boxes.** The workspace has no cards inside cards and no decorative containers.
5. **Detail appears on selection, in place.** Nothing is open permanently: no worker pane, no log drawer.
6. **Colour is a signal, not decoration.** Sage means verified, amber means waiting on a human, brick means failed, graphite means in motion. Colour only appears on 6px dots, strokes and marks. Text stays charcoal or stone, and there are no pills or badges.
7. **Every number comes from an observed record.** If there's no evidence, show "—" or "Not observed", never a healthy default. No scores and no percentages.
8. **Each screen has at most one filled (charcoal) button**, and it is that screen's decision: Send, an approval, or Merge.

## 3. Design tokens and geometry

**Typography.** The UI font is Inter (SF Pro as fallback) with tabular numerals. Monospace is JetBrains Mono, used for SHAs, file paths, code and the radar's engraved labels. Only weights 400, 500 and 600 are used.

| Role | Size and line height | Weight | Use |
|---|---|---|---|
| Title | 16/22 | 600 | Only the selected change title in Reviews and the task title in a detail view |
| Reading | 15/24 | 400 | Conversation messages |
| Body | 14/20 | 400 or 500 | Navigation, list titles, goal line, buttons |
| Secondary | 13/18 | 400 | Current focus, summaries, check line |
| Metadata | 12/16 | 400 | Times, agent names, SHAs, radar labels (mono 12, uppercase, 0.08em tracking for engraving) |

Nothing in the product chrome is larger than 16px.

**Shell geometry (1440×900 viewport)**

- **Sidebar** is 184px wide.
  - The top 44px is the traffic-light band, aligned with the top bar.
  - The project switcher row starts at y=52 and is 32px tall: a 20px monogram tile, the name at 14/500 and a chevron.
  - Three navigation rows start at y=96. Each is 32px tall, with 8px side insets, 16px icons at a 1.5 stroke and 10px between icon and label.
  - Settings is pinned 12px from the bottom.
- **Top bar** is 44px tall with a 1px bottom hairline. Its content starts 24px from the sidebar edge.
- **Workspace** is 1256×856 with 24px horizontal padding. Spacing uses a 4px grid with the steps 4, 8, 12, 16, 24 and 32.
- **Controls:** buttons are 32px tall with 14px horizontal padding. The segmented control has a 28px track. List rows are 52px (two lines). Controls use a 6px radius; frames and the composer use 10px.

**Surfaces and colours**

| Token | Hex | Use |
|---|---|---|
| canvas | `#F7F5F0` | Workspace and top bar |
| sidebar | `#EFEBE4` | Sidebar |
| raised | `#FFFDF9` | Composer, diff pane, popovers, selected segment |
| inset | `#ECE8E0` | Preview stage, segmented track, user message bubble |
| hairline | `#E1DCD2` | Dividers |
| line-strong | `#CCC6BA` | Secondary button borders, radar rings |
| text | `#24221F` | Primary text |
| text-2 | `#5F5A52` | Secondary text |
| text-3 | `#8F897E` | Tertiary text |
| hover | `#EAE6DE` | Hover fill |
| selected | `#E5E0D6` | Selected and active fill |
| sage | `#6E8B6F` | Verified |
| amber | `#B97A2E` | Waiting on a human |
| brick | `#A9503A` | Failed |
| graphite | `#3B3833` | In motion |
| diff-add | `#E7EEDF` | Added lines |
| diff-del | `#F3E2DA` | Removed lines |

The only shadow is on popovers and sheets: `0 8px 24px rgba(36,34,31,.10)`.

**Active, focus and hover treatment**

- **Active navigation or list row:** `selected` fill with a 6px radius, text at 500 weight and a charcoal icon. There is no accent bar.
- **Hover:** `hover` fill.
- **Keyboard focus:** a 2px solid `#24221F` outline with a 2px canvas-coloured offset. On the radar, focus is a 2px ring with a 3px gap around the mark.
- **Primary button:** `#24221F` fill with `#FFFDF9` text.
- **Secondary button:** 1px `line-strong` border on a transparent fill.

## 4. Screen specifications

### Conversation

**Composition**
- One 680px reading column, centred in the workspace (x=472 to 1152).
- Messages start 32px below the top bar.
- The composer is docked 24px from the bottom: 680px wide, at least 96px tall, `raised` surface, 1px hairline, 10px radius.

**Visible content**
- **Lead agent messages** have no bubble. Each has a 16px Factory mark, "Factory" at 13/500 and the time at 12px, followed by the body at 15/24.
- **User messages** are right-aligned in an `inset` bubble, at most 480px wide with a 12px radius.
- **A lead message that hands off a change** carries one reference row between hairlines: "Tighten Collection grid spacing · #24 · Open in Reviews →".
- **A lead message that asks for approval** carries two 28px secondary buttons.

**Primary action:** Send, a 28px charcoal circle.

**Progressive disclosure**
- "Show 5 tasks" under the delegation message expands a list of task titles and assignees at 13px.
- The goal chevron in the top bar opens a 420px popover with the saved goal and an Edit option.

**Excluded:** change list, preview, diff, merge controls, worker transcripts, model or provider pickers, metrics and empty-state help copy.

### Progress, with Radar selected

**Composition**
- A Graph | Board | Radar segmented control sits on the right of the top bar, left of the factory state. It is 28px tall, 13px, with Radar selected.
- The workspace splits into an instrument area (872px) and an index column (384px, with a 1px hairline on its left).
- The dial is 600px in diameter, centred at x=620, y=472.
- A single legend line sits 16px below the dial at 12px tertiary: "Distance from centre = time in stage".

**Index column**
This column answers "who's on what" and is a list of work, not changes. It has three groups, each with a 12px label:
- **Working:** "Records carousel Reduce Motion", then "Codex · 14m · editing RecordShelf.swift". Also "Venue detail map link", then "Claude · 52m · running checks".
- **Needs you:** an amber dot, "Refresh media cache", then "waiting for network approval · 1h 10m".
- **Up next:** "Calendar band contrast · ready · 3h" and "Recap print footer spacing · backlog · 1d+".

The column ends with one text line: "2 changes waiting in Reviews →".

**Primary action:** none by default, because this is a monitoring screen. When a blocked task is selected, "Allow once" becomes the single filled button.

**Progressive disclosure**
- Hovering a mark shows its label and a metadata line.
- Selecting a mark or a row swaps the index column for task detail, with a back arrow. Detail shows the title at 16/600, the acceptance criteria (up to three lines), and the latest run: provider, start time, last heartbeat and the five most recent events in mono at 12px. It also offers "Steer…" (opens Conversation with the task referenced) and "Interrupt" (secondary).
- Graph and Board are alternative views of the same tasks and runs, using the same index column.

**Excluded:** change list, preview pane, diff, merge controls, KPI tiles, sparklines, percentages, scores, deployment "health" without observation, and any persistent log pane.

### Reviews

**Change list (left rail, 264px)**
- Groups with 12px labels: "Ready" (2), "Changes requested" (1), and "Merged" (3, collapsed).
- Each row is 52px: a 6px status dot, the title at 14/500 (truncated), then metadata at 12px, e.g. "#24 · Codex · checks passed".

**Main area (992px, 20px padding)**
- **Header, 64px.**
  - First line: the title at 16/600 and "#24" in tertiary. On the right, a Split | Preview | Diff segmented control with Split selected.
  - Second line: a one-sentence agent summary at 13px text-2 with "More". On the right, `a4f3c2e` in 12px mono and "Open pull request ↗".
- **Stage, 952×704 (about 70% of the viewport).** In Split mode:
  - A 400px `inset` preview stage holds a 320×692 iPhone screen (390×844 at 0.82 scale), centred with a 1px hairline and a soft shadow.
  - A 540px `raised` diff pane shows a 36px file header (path in 12px mono, "+18 −11", and a "3 files ▾" switcher), then code at 13/20 mono with add and delete tints.
  - Preview mode or Diff mode gives the whole stage to one side.
- **Footer, 56px.**
  - Left: a sage dot with "6 checks passed on a4f3c2e" and a "Details" link, then an amber dot with "1 limitation".
  - Right: "Request changes" (secondary) and "Merge" (primary).

**Primary action:** Merge.

**Progressive disclosure**
- Details opens a popover listing each check with its command, exit code and duration.
- The limitation link discloses its note: "No accessibility-size screenshot captured".
- "More" expands the agent summary in place.
- Request changes replaces the footer with a 120px inline note field.
- Merge opens a confirm sheet bound to the tested SHA: "Merge a4f3c2e into main".

**Excluded:** radar, graph, conversation transcript, worker logs and any metrics.

## 5. Radar concept: "Sundial"

**Signature.** The dial is a drafted paper instrument: hairline engraved rings and a graduated bezel on warm stone, with no glow, no sweep line and no dark-mode sci-fi styling. The distinctive gesture is the **gnomon shadow**. The lead agent sits at the centre and casts a tapering graphite wedge toward each task it has delegated an active run to. Shadows show who is working on what, and a shadow's length shows how long that work has run. Waiting work drifts outward toward the rim, so the dial's edge is where attention belongs.

**How real state maps to geometry**

- **Angle is the task's status**, in four fixed quadrants running clockwise, so work literally travels around the dial:
  - Queued (backlog and ready): 9 to 12 o'clock.
  - Running: 12 to 3.
  - Review: 3 to 6.
  - Done in the last 24h: 6 to 9.
  - Labels are engraved at the rim in 12px uppercase mono.
- **Radius is the time since the task entered its current status**, on a log scale from 64px (just entered) to 290px.
  - Rings sit at 15m, 1h, 4h and 1d, labelled along the 12 o'clock axis.
  - Anything older than 1d is pinned to the rim with a short outward tick.
  - The band beyond 4h is shaded slightly darker (`#EAE5DC`), and marks inside it are always labelled.
- **Marks:**
  - Queued: hollow 8px circle.
  - Running: filled graphite.
  - Review: charcoal ring with a filled centre, meaning "yours".
  - Done: faint 5px dot, unlabelled.
  - Blocked or needs approval: amber mark, with an amber wedge hatched at 45°.
  - Placement within a quadrant is a stable hash of the task ID plus collision avoidance, so marks don't jump.
- **Centre:** a 44px disc reading "Lead", with its state underneath at 12px ("planning" or "idle").
- **Gnomon shadows:** one per active run, drawn from the centre disc to the task mark. Each tapers from 10px to 2px, filled graphite at 18% opacity. They follow the parent (delegation) links between tasks.
- **Dependencies:** dotted 1px chords between marks, drawn only for prerequisite links, never inferred from layout.
- **Bezel:** a 10px band at r=290–300. Each quadrant's bezel is its own 24-hour timeline running clockwise: every status change into that quadrant leaves a 1px tick, fading with age. Dense ticks mean real throughput, with no number attached.

**Motion**
- No continuous animation.
- On each observed heartbeat event, the shadow tip brightens from 18% to 32% opacity and back over 800ms.
- On a status change, the mark travels clockwise along its arc to the new quadrant over 500ms with ease-in-out, and its bezel tick appears.
- Radius (time in status) updates every 30s, so the drift is imperceptible.

**Reduced motion**
- No pulses and no travel; marks move instantly.
- Heartbeat freshness is shown statically: a solid shadow tip means a heartbeat in the last 2 minutes, a hollow tip means stale.

**When disconnected**
- The dial desaturates to stone and shadows turn dashed.
- The top bar reads "Disconnected · last snapshot 3m ago".

There is no progress percentage, health score or throughput number anywhere on the dial.

## 6. Image prompts

### Conversation

```text
Realistic flat UI screenshot of a macOS desktop application window, exactly 1440x900 pixels, edge to edge, no device frame, no perspective, no outer shadow. Light theme. Crisp Inter typeface; JetBrains Mono only for hashes and code. Palette: workspace canvas warm off-white #F7F5F0, sidebar pale stone #EFEBE4, raised surfaces warm white #FFFDF9, inset fills #ECE8E0, 1px hairlines #E1DCD2, text charcoal #24221F, secondary #5F5A52, tertiary #8F897E; sage #6E8B6F and amber #B97A2E appear only as tiny 6px dots. Calm, quiet, precise, generous but compact. No pills, no badges, no cards inside cards, no gradients, no emoji, no illustrations, no headings larger than 16px.

APP SHELL: Left sidebar 184px wide, full height, pale stone, 1px hairline on its right edge. Top 44px of sidebar holds macOS red/yellow/green traffic lights at left. Below them a project switcher row: 20px rounded-square stone tile with a small charcoal "G", then "Green Room" 14px medium, then a tiny chevron. 12px gap, then three 32px navigation rows with 16px thin line icons and 14px labels: "Conversation" (speech bubble icon), "Progress" (three connected nodes icon), "Reviews" (stacked layers icon) with a plain tertiary numeral "2" right-aligned, not a badge. Active row has a #E5E0D6 fill with 6px radius and charcoal medium text; inactive rows are #5F5A52. At the sidebar bottom: "Settings" with a gear icon. Nothing else in the sidebar: no wordmark, no project list, no stats. Top bar 44px tall across the workspace, canvas colored, 1px hairline bottom. At its left, 24px in: "Make the Collection faster to browse" 14px medium charcoal with a tiny down chevron, a middle dot, then "Now: Records carousel respects Reduce Motion" 13px secondary grey. At its right: a 6px graphite dot, "Running · 2 agents" 13px secondary, then a 28px ghost square pause icon button.

THIS SCREEN: "Conversation" is the active nav row. The workspace contains a single centered reading column 680px wide; large calm margins of empty canvas on both sides. Messages, top to bottom:
1. Right-aligned user bubble, #ECE8E0 fill, 12px radius, 15px text: "This week, focus on Collection. The grid feels loose and the carousel ignores Reduce Motion." Small "You · 9:42" above it in 12px tertiary.
2. Lead agent message without bubble: a 16px minimal charcoal square-mark icon, "Factory" 13px medium, "9:44" 12px tertiary; body 15px charcoal: "Got it. I saved the goal as "Make the Collection faster to browse" and split it into five tasks. Codex is on the carousel now; Claude is adding the map link on venue detail." Under it a small text disclosure "Show 5 tasks ›" in 13px secondary.
3. Lead agent message, "11:05": "Grid spacing is ready for your review. Checks passed on a4f3c2e. I couldn't capture an accessibility-size screenshot, so that's listed as a limitation." Directly under it a single quiet reference line between two hairlines: a small layers icon, "Tighten Collection grid spacing" 14px, "#24" tertiary, and at the right "Open in Reviews →" 13px charcoal underlined-on-hover link. No thumbnail, no preview.
4. Lead agent message, "11:58": "Refreshing the media cache needs network access to Supabase. Allow it for this run only?" Under it two small 28px secondary outline buttons: "Allow once" and "Skip task".
Docked 24px above the bottom, aligned to the 680px column: a composer, warm white #FFFDF9, 1px hairline border, 10px radius, about 96px tall, placeholder "Tell Factory what matters next" 14px tertiary, and a 28px charcoal circular send button with a white up-arrow at bottom right.
Must NOT include: any list of changes, PR preview pane, app screenshot, diff, checks panel, merge or request-changes buttons, worker list, model picker, charts or metrics.
```

### Progress (Radar)

```text
Realistic flat UI screenshot of a macOS desktop application window, exactly 1440x900 pixels, edge to edge, no device frame, no perspective, no outer shadow. Light theme. Crisp Inter typeface; JetBrains Mono for small engraved labels. Palette: workspace canvas warm off-white #F7F5F0, sidebar pale stone #EFEBE4, raised surfaces warm white #FFFDF9, inset fills #ECE8E0, 1px hairlines #E1DCD2, text charcoal #24221F, secondary #5F5A52, tertiary #8F897E; graphite #3B3833; sage #6E8B6F and amber #B97A2E only as small marks and strokes. Calm, quiet, precise, compact. No pills, no badges, no cards inside cards, no glow, no neon, no dark mode, no headings larger than 16px.

APP SHELL: Left sidebar 184px wide, full height, pale stone, 1px hairline on its right edge. Top 44px of sidebar holds macOS red/yellow/green traffic lights at left. Below them a project switcher row: 20px rounded-square stone tile with a small charcoal "G", then "Green Room" 14px medium, then a tiny chevron. 12px gap, then three 32px navigation rows with 16px thin line icons and 14px labels: "Conversation" (speech bubble icon), "Progress" (three connected nodes icon), "Reviews" (stacked layers icon) with a plain tertiary numeral "2" right-aligned, not a badge. Active row has a #E5E0D6 fill with 6px radius and charcoal medium text; inactive rows are #5F5A52. At the sidebar bottom: "Settings" with a gear icon. Nothing else in the sidebar: no wordmark, no project list, no stats. Top bar 44px tall across the workspace, canvas colored, 1px hairline bottom. At its left, 24px in: "Make the Collection faster to browse" 14px medium charcoal with a tiny down chevron, a middle dot, then "Now: Records carousel respects Reduce Motion" 13px secondary grey. At its right: a 6px graphite dot, "Running · 2 agents" 13px secondary, then a 28px ghost square pause icon button.

THIS SCREEN: "Progress" is the active nav row. In the top bar, just left of "Running · 2 agents", a compact segmented control 28px tall on an #ECE8E0 track with three 13px segments "Graph", "Board", "Radar"; "Radar" is selected with a warm white raised segment.
The workspace splits into a large left instrument area (about 872px wide) and a right index column 384px wide separated by a single 1px vertical hairline.
LEFT: a circular instrument called a sundial radar, 600px diameter, centered in the left area, drawn like an engraved drafting instrument on warm paper. Four fine concentric hairline rings (#CCC6BA) labeled along the vertical 12 o'clock axis in 12px mono tertiary: "15m", "1h", "4h", "1d". The outermost band between the 4h ring and rim is shaded slightly darker stone (#EAE5DC). The rim is a 10px graduated bezel of thin 1px tick marks of varying density and fading opacity, like a seismograph wrapped around a dial. Two thin hairline radii split the circle into four quadrants. Engraved uppercase 12px mono labels with wide letter spacing just outside the rim at each quadrant center: top-left "QUEUED", top-right "RUNNING", bottom-right "REVIEW", bottom-left "DONE 24H". At the exact center a 44px charcoal disc with "Lead" in small warm white text and "planning" in 12px tertiary just below it.
Marks with 12px charcoal labels and 12px tertiary metadata:
- RUNNING quadrant: filled graphite dot "Carousel motion" near the 15m ring, meta "Codex · 14m"; filled graphite dot "Map link" just inside the 1h ring, meta "Claude · 52m". From the center disc to each of these two dots runs a soft tapering graphite wedge-shaped shadow, about 10px wide at the center narrowing to 2px at the dot, at low opacity — like a sundial's gnomon shadow. An amber dot "Media cache" between the 1h and 4h rings with meta "needs approval · 1h 10m" and its own wedge shadow in pale amber with fine diagonal hatching.
- REVIEW quadrant: two charcoal ring marks with filled centers: "Grid spacing" near the 15m ring and "Venue search" between 1h and 4h. No shadows to these.
- QUEUED quadrant: hollow circle "Band contrast" near the 4h ring; hollow circle "Footer spacing" pinned on the rim with a short outward tick.
- DONE 24H quadrant: three tiny faint unlabeled dots.
- One thin dotted chord connects "Map link" to "Carousel motion" (dependency).
Below the dial, left-aligned to it, one 12px tertiary line: "Distance from centre = time in stage".
RIGHT index column, 24px padding, plain list, no cards: 12px tertiary label "Working"; row: small graphite dot, "Records carousel Reduce Motion" 14px, under it "Codex · 14m · editing RecordShelf.swift" 12px tertiary; row: "Venue detail map link", "Claude · 52m · running checks". 12px tertiary label "Needs you"; row with amber dot: "Refresh media cache", "waiting for network approval · 1h 10m". 12px tertiary label "Up next"; rows with hollow dots: "Calendar band contrast · ready · 3h", "Recap print footer spacing · backlog · 1d+". At the bottom of the column a single 13px text link: "2 changes waiting in Reviews →".
Must NOT include: any list of changes or PRs, preview pane, app screenshot, diff, merge or request-changes buttons, KPI tiles, sparklines, line charts, percentages, scores, "healthy" labels, big numbers.
```

### Reviews

```text
Realistic flat UI screenshot of a macOS desktop application window, exactly 1440x900 pixels, edge to edge, no device frame around the window, no perspective, no outer shadow. Light theme. Crisp Inter typeface; JetBrains Mono for code and hashes. Palette: workspace canvas warm off-white #F7F5F0, sidebar pale stone #EFEBE4, raised surfaces warm white #FFFDF9, inset fills #ECE8E0, 1px hairlines #E1DCD2, text charcoal #24221F, secondary #5F5A52, tertiary #8F897E; sage #6E8B6F and amber #B97A2E only as tiny dots; diff added lines #E7EEDF, removed lines #F3E2DA. Calm, quiet, precise, compact. No pills, no badges, no cards inside cards, no gradients, no headings larger than 16px in the app chrome.

APP SHELL: Left sidebar 184px wide, full height, pale stone, 1px hairline on its right edge. Top 44px of sidebar holds macOS red/yellow/green traffic lights at left. Below them a project switcher row: 20px rounded-square stone tile with a small charcoal "G", then "Green Room" 14px medium, then a tiny chevron. 12px gap, then three 32px navigation rows with 16px thin line icons and 14px labels: "Conversation" (speech bubble icon), "Progress" (three connected nodes icon), "Reviews" (stacked layers icon) with a plain tertiary numeral "2" right-aligned, not a badge. Active row has a #E5E0D6 fill with 6px radius and charcoal medium text; inactive rows are #5F5A52. At the sidebar bottom: "Settings" with a gear icon. Nothing else in the sidebar: no wordmark, no project list, no stats. Top bar 44px tall across the workspace, canvas colored, 1px hairline bottom. At its left, 24px in: "Make the Collection faster to browse" 14px medium charcoal with a tiny down chevron, a middle dot, then "Now: Records carousel respects Reduce Motion" 13px secondary grey. At its right: a 6px graphite dot, "Running · 2 agents" 13px secondary, then a 28px ghost square pause icon button.

THIS SCREEN: "Reviews" is the active nav row. Workspace = narrow change rail on the left (264px, 1px hairline on its right) and a large review area filling the rest.
RAIL: 12px tertiary label "Ready". Row 1 selected with #E5E0D6 fill, 6px radius, 52px tall: sage 6px dot, "Tighten Collection grid spacing" 14px medium, under it "#24 · Codex · checks passed" 12px tertiary. Row 2: sage dot, "Venue search empty state", "#23 · Claude · checks passed". Label "Changes requested"; row: amber dot, "Share sheet button label", "#22 · Codex · updated". At the bottom a collapsed 13px tertiary line "Merged · 3 ›".
REVIEW AREA, 20px padding. Header line 1: "Tighten Collection grid spacing" 16px semibold charcoal, "#24" tertiary beside it; far right a 28px segmented control on #ECE8E0 track with "Split", "Preview", "Diff", "Split" selected. Header line 2: 13px secondary "Reduces Grid card gutters from 20 to 12 points and aligns show counts to the artist name baseline." followed by "More"; far right "a4f3c2e" in 12px mono tertiary and "Open pull request ↗" 13px.
Below, the stage fills most of the screen (about 952x704):
LEFT 400px: an inset #ECE8E0 stage with a single iPhone screen, about 320x692, centered, thin hairline edge and very soft shadow, no hardware bezel. Inside it the Green Room iOS app Collection screen on warm paper background, Space Grotesk typeface, deep forest green accents: compact header "Collection", a small "Records | Grid" toggle with Grid selected, a scope row "Artists  Venues  Shows" with Artists selected, then a two-column grid of record cards with tight even gutters: each card is a square photographic album-style sleeve (moody concert and portrait photography) with a black grooved vinyl disc peeking out of its right edge, the disc's center label a small circular crop of the same photo; below each card the artist name in 13px and "4 shows" style counts in small muted text. Artists: "Moss Avenue", "Hollie Tan", "The Lantern Choir", "Kite Rivers", "Ezra Cole", "Juniper Hall". A minimal bottom tab bar.
RIGHT about 540px: warm white diff pane, 1px hairline, 10px radius. 36px file header: "Features/Collections/ArtistRecordCard.swift" 12px mono, "+18 −11" tertiary, and at right "3 files ▾". Below, 13px monospace Swift code with line numbers in tertiary gutter; a few removed lines tinted #F3E2DA such as ".padding(.horizontal, 20)" and "spacing: 20", added lines tinted #E7EEDF such as ".padding(.horizontal, 12)", "spacing: 12", ".alignmentGuide(.firstTextBaseline)". Realistic, short, legible.
FOOTER bar 56px under the stage, separated by a hairline: left side a sage 6px dot and "6 checks passed on a4f3c2e" 13px with a "Details" link, then an amber 6px dot and "1 limitation" 13px link. Right side: secondary outline button "Request changes" and primary charcoal filled button "Merge" with warm white text, both 32px tall.
Must NOT include: radar, graph, progress strip, conversation transcript, worker logs, metrics, extra panels.
```

## 7. Implementation handoff

- **Shell.** Use a CSS grid with a 184px column and a 44px top bar. Store the tokens above as one shared file used by both the Tauri desktop app and the TUI's theme. The TUI keeps the same tree: three destination tabs on keys 1/2/3, the goal and focus in the header, and Radar rendered as quadrant lists sorted by time in stage with small age bars.
- **Radar build.** Draw Sundial in SVG first: d3 `scaleLog` for radius (1 minute to 1 day, clamped), fixed quadrant angles, a stable hash of the task ID plus `forceCollide` and `forceRadial` for placement. Move to Canvas only if there are more than about 300 marks.
- **Radar data needs.** Time-in-status needs a "status entered at" value derived from the events table, because `Task.updated_at` changes for other reasons. Shadows need active `Run` records with their provider and heartbeat or lease-renewal events. Tethers come from `parent_id` and chords from `depends_on`. Bezel ticks come from status-transition events in the last 24h.
- **Uncertainty: blocked tasks.** A task blocked by a `needs_approval` run keeps the angle of the quadrant it stalled in. This assumes the store records the previous status.
- **Uncertainty: iOS previews.** Reviews assumes the review bundle includes simulator screenshots. If none was captured, the preview side shows "No preview captured" and the stage defaults to Diff.
- **Uncertainty: badge counts.** The Reviews count should come from tasks in Review status, reconciled against the observed GitHub PR state.
- **Uncertainty: rendered images.** Image models may not draw the dial or the Swift code accurately. Judge the next set on composition, hierarchy and the exclusions, not on exact mark placement.
- **Not specified here.** This brief only defines Graph and Board as the other Progress views sharing the same index column. Their visuals need a follow-up pass in this same visual language.