## 1. Corrected Radar concept: Sundial

The Radar now shows only active work: planning, building, checking, and work waiting on a human input. Reviews stays a separate sidebar destination, and the Radar body names no change, PR or review item.

**Signature.** The dial is still a drafted paper instrument: engraved hairline rings, a static graduated bezel and engraved mono labels, with no glow and no sweep. The lead agent sits at the centre and casts a tapering graphite gnomon shadow to each piece of work an agent holds. Older work sits nearer the rim. The rim is where a blocked input appears.

**Which work appears.** A task appears only if it is `Running`, or `Blocked` with a retained run. Backlog, ready, review and done tasks are never drawn, and there are no counts, links or queue anywhere on the dial.

**Angle is the current stage**, in four fixed quadrants running clockwise:
- **PLAN (9 to 12 o'clock):** a planning-kind task with an active run.
- **BUILD (12 to 3):** an implementation-kind task with an active run and no supervisor check in progress.
- **CHECK (3 to 6):** a supervisor-run check has started and has no result yet.
- **WAITING (6 to 9):** the run is `needs_approval`, or the task is `Blocked` with a recorded reason.

A mark's position inside its quadrant comes from a stable hash of the task ID, so marks never reshuffle.

**Radius is time in the current stage.** It is measured from the event that entered that stage: run start, check start, or approval request. The scale is logarithmic from 64px to 290px, with engraved rings at 15m, 1h and 4h. At 1 day or more, the mark is pinned to the rim with a short outward tick.

**The shadow is the radius, drawn.** It runs from the lead disc's edge to the mark, so its length is exactly the time in current stage, never total run duration.
- Shadows appear only while a run holds a live lease.
- A waiting mark gets a pale amber, hatched shadow.
- The shadow's taper is purely visual and encodes nothing.

**Shadow tip.** A solid tip means a heartbeat was observed in the last 2 minutes. A hollow tip means the lease is stale, which is shown as stale, not as waiting.

**Blocked input.** Every WAITING mark extends a thin amber line to a notch on the bezel. The notch is labelled with the recorded input in 12px, for example "network approval".

**Labels.** Each mark gets a short task title and one metadata line, "Codex · 14m". That time is the same number the radius encodes.

**Centre disc.** It reads "Lead", with the lead's own observed run state underneath: "planning", "idle", or "—" when there is no evidence.

**Empty and disconnected states.** An empty quadrant keeps only its engraved label. With no active work, the centre reads "idle". When disconnected, the dial desaturates and shadows turn dashed.

**Index column.** It lists only the same marks, in two groups: "Working" and "Needs you". Selecting a mark or a row opens task detail in place.

**Motion.** There is no continuous animation.
- On each observed heartbeat, the shadow tip brightens over 800ms.
- On a stage change, the mark travels clockwise along the arc over 500ms, and its radius resets toward the centre.
- Radius re-renders every 30s.

**Reduced motion.** There are no pulses and no travel; marks reposition instantly, and freshness shows only as a solid or hollow tip.

**Removed.** The Review and Done quadrants, the queued marks, dependency chords, bezel history ticks, the "Up next" group and the "2 changes waiting in Reviews" line are all gone.

## 2. Revised Radar image prompt

```text
Realistic flat UI screenshot of a macOS desktop application window, exactly 1440x900 pixels, edge to edge, no device frame, no perspective, no outer shadow. Light theme. Crisp Inter typeface; JetBrains Mono for small engraved labels. Palette: workspace canvas warm off-white #F7F5F0, sidebar pale stone #EFEBE4, raised surfaces warm white #FFFDF9, inset fills #ECE8E0, 1px hairlines #E1DCD2, text charcoal #24221F, secondary #5F5A52, tertiary #8F897E; graphite #3B3833; sage #6E8B6F and amber #B97A2E only as small marks and strokes. Calm, quiet, precise, compact. No pills, no badges, no cards inside cards, no glow, no neon, no dark mode, no headings larger than 16px.

APP SHELL: Left sidebar 184px wide, full height, pale stone, 1px hairline on its right edge. Top 44px of sidebar holds macOS red/yellow/green traffic lights at left. Below them a project switcher row: 20px rounded-square stone tile with a small charcoal "G", then "Green Room" 14px medium, then a tiny chevron. 12px gap, then three 32px navigation rows with 16px thin line icons and 14px labels: "Conversation" (speech bubble icon), "Progress" (three connected nodes icon), "Reviews" (stacked layers icon) with a plain tertiary numeral "2" right-aligned, not a badge. Active row has a #E5E0D6 fill with 6px radius and charcoal medium text; inactive rows are #5F5A52. At the sidebar bottom: "Settings" with a gear icon. Nothing else in the sidebar: no wordmark, no project list, no stats. Top bar 44px tall across the workspace, canvas colored, 1px hairline bottom. At its left, 24px in: "Make the Collection faster to browse" 14px medium charcoal with a tiny down chevron, a middle dot, then "Now: Records carousel respects Reduce Motion" 13px secondary grey. At its right: a 6px graphite dot, "Running · 2 agents" 13px secondary, then a 28px ghost square pause icon button.

THIS SCREEN: "Progress" is the active nav row. In the top bar, just left of "Running · 2 agents", a compact segmented control 28px tall on an #ECE8E0 track with three 13px segments "Graph", "Board", "Radar"; "Radar" is selected with a warm white raised segment.
The workspace splits into a large left instrument area (about 872px wide) and a right index column 384px wide, separated by a single 1px vertical hairline. Lots of calm empty paper around the instrument.
LEFT: a circular instrument, 600px diameter, centered in the left area, drawn like an engraved drafting instrument printed on warm paper. Three fine concentric hairline rings (#CCC6BA) labeled along the vertical 12 o'clock axis in 12px mono tertiary: "15m", "1h", "4h". The rim is a thin 10px static graduated bezel of evenly spaced fine 1px engraved ticks, every fifth tick slightly longer, all the same light stone tone. Two thin hairline radii divide the circle into four quadrants. Engraved uppercase 12px mono labels with wide letter spacing just outside the rim at each quadrant's center: top-left "PLAN", top-right "BUILD", bottom-right "CHECK", bottom-left "WAITING". At the exact center a 44px charcoal disc with "Lead" in small warm white text, and "idle" in 12px tertiary just below it.
Only three marks on the whole dial, each with a 12px charcoal label and a 12px tertiary meta line:
- BUILD quadrant: a filled graphite 8px dot just inside the 15m ring, labeled "Carousel motion", meta "Codex · 14m". A soft tapering graphite wedge-shaped shadow runs from the edge of the center disc exactly to this dot, about 10px wide at the center narrowing to 2px, low opacity, like a sundial's gnomon shadow. Its tip is a solid dot.
- CHECK quadrant: a filled graphite dot slightly closer to the center than the first, labeled "Map link", meta "Claude · 9m", with the same style of tapering graphite shadow from the center disc to it, solid tip.
- WAITING quadrant: an amber 8px dot between the 1h and 4h rings, labeled "Media cache", meta "1h 10m". Its shadow from the center disc is pale amber with fine diagonal hatching. From the amber dot a thin amber hairline continues outward to a small amber notch on the bezel, labeled just outside the rim in 12px amber-tinted text "network approval".
- PLAN quadrant is empty except for its engraved label.
Below the dial, left-aligned to it, one 12px tertiary line: "Distance from centre = time in current stage".
RIGHT index column, 24px padding, plain list, no cards: 12px tertiary label "Working"; row: small graphite dot, "Records carousel Reduce Motion" 14px charcoal, under it "Codex · build · 14m" 12px tertiary; row: small graphite dot, "Venue detail map link", under it "Claude · check · 9m". 16px gap, 12px tertiary label "Needs you"; row: amber dot, "Refresh media cache" 14px, under it "Waiting for network approval · 1h 10m" 12px tertiary, and beneath it two small 28px outline buttons "Allow once" and "Skip task". Nothing else in the column; empty paper below.
The workspace body must NOT include: any mention of reviews, changes, pull requests, ready or done items, queues, counts, links to other screens, previews, app screenshots, diffs, merge or request-changes buttons, KPI tiles, sparklines, line charts, percentages, scores, health labels, or big numbers.
```

## 3. Conversation and Reviews prompts

Both prompts still apply unchanged, except for one correction in Reviews. Its stage should be 952×696 (previously "about 952x704"), so the 856px workspace adds up: 40px total padding, a 64px header, the 696px stage and a 56px footer. The 320×692 phone preview still fits within the 696px stage.