# Original Claude authoring handoff

This records Claude's initial public handoff before parent review fixes. The current [QA record](../../../apps/desktop/design-qa.md) supersedes its statements about run persistence, context capture, keyboard access, and final verification.

I'll read the required docs, visual sources, and skills first, then write the frontend.Now the three skill files.I have the context. Checking which Phosphor icon names exist before writing code.I've read all sources. Writing the frontend now: fixtures in `data.js`, shared controls in `ui.jsx`, then the shell and pages.Now the shell, conversation, files and settings in `App.jsx`.Moving `ApprovalActions` into `ui.jsx` to avoid a circular import, and cleaning a stray export.Now `Runs.jsx`: list, timeline, run detail with tabs, and the retry dialog.Now `Progress.jsx` with Graph, Board and the canvas Sundial radar.Now `Reviews.jsx`.Fixing a label overlap in the timeline axis (the 12:00 hour label collides with the Now marker), then writing the stylesheet.Updating the page title, then running a quick esbuild bundle (output to `/dev/null`) to catch syntax and import errors.I've built the clickable design preview. The only check I ran is an esbuild bundle of `src/main.jsx` (output to `/dev/null`): it compiles and every import resolves, including the Phosphor icon names. I haven't run `vite build`, opened it in a browser, measured layouts or tested keyboard and screen-reader behaviour; that's your QA pass.

**Files written** (all in `apps/desktop/`)
- `index.html`: page title only.
- `src/data.js`: the sample Green Room data on a fixed 12:00 clock, the model catalog, settings defaults, and small helpers for model/effort checks, file context rules, task state and radar stage.
- `src/ui.jsx`: shared pieces: tabs with arrow-key support, a dialog built on the native `<dialog>` (traps focus, closes on Escape, restores focus), status labels, status notices, the model/effort/mode selects and the approval buttons.
- `src/App.jsx`: the app shell, routing, all state and saving, plus the Conversation, Files and Settings pages.
- `src/Runs.jsx`: run list, timeline, run detail and the retry dialog.
- `src/Progress.jsx`: Graph, Board and the canvas Radar with its work list.
- `src/Reviews.jsx`: the Reviews workspace.
- `src/styles.css`: the warm tokens from `tokens.json` and all layout. Budget: 168px sidebar, 40px header, 16px padding, 28–32px controls, 32px nav rows, 14/22 body, 13px metadata, 16px headings.

**Main interaction choices**
- **Routing:** URL hash links such as `#/runs/R-8K4D`, so sidebar items are real links and a direct run link opens that run. Back returns to the list with filters kept and focuses the row you came from. Opening a run focuses its heading.
- **Model picker:** a compact "Next run" button beside Send shows the resolved model, effort and mode, marked Default or Override. It opens an inline panel with three native selects, each with a "Use project default (…)" option. I chose this over three always-visible selects because those options are long enough to crowd an 820px composer.
- **Picker rules:** Gemini 3.1 is disabled as not connected. Max effort is unsupported on GPT-5.3; picking it shows an error and disables Send, with no silent fallback. The override clears after Send, and the notice says so.
- **Send:** adds your message and a queued run (IDs `R-P001`, `R-P002`, …) with a fixed settings snapshot, plus a note that no model was called. There's no AI reply.
- **Retry:** only for failed, interrupted or cancelled runs, and only when the task has no active run. The dialog starts from the original run's settings, offers "Current project defaults", and counts any edit as a retry edit. It creates one new run, guarded against double clicks.
- **Interrupt:** shows Interrupting… for about 1.2 seconds, then Interrupted, and creates no new run. Allow once and Skip task are only noted locally; the run stays Needs approval.
- **Saved data:** project settings, the composer draft and override, and file edits are saved in the browser, versioned, with a fallback if the saved data can't be read. Runs and review decisions reset on reload.
- **Radar:** canvas draws the rings, bezel ticks, tapered graphite shadows, the hatched amber waiting shadow and the rim notch. Marks and labels are buttons on top. Distance uses straight-line segments between the 15m, 1h and 4h rings; diameter is capped at 640px. There's no animation. Below 1024px the work list moves under the dial.
- **Reviews:** the queue is 208px and becomes a dropdown at 1024px and below. Split, Preview and Diff tabs. The screenshot keeps its proportions (`object-fit: contain`). Merge preview and Request changes go through dialogs labelled as simulated.
- **Files:** changing roots, ignore patterns or a file's include setting saves immediately and only affects future runs. Settings uses an explicit Save/Cancel draft instead.

**Gaps**
- I didn't add steering, delivery-failure or reconnect/stale-connection states; acceptance case 8 isn't covered.
- Only the #24 review has a real screenshot. #23 and #22 honestly show "missing" or "text-only" placeholders.
- "Open sample" artifact links open the sample PNG in a new tab.
- The project switcher is a native select with one project; other projects are a disabled option.
- The radar's Lead centre always reads "idle", and there's no disconnected state.
- Two marks landing close together in the same quadrant could overlap labels; the fixtures avoid this.
- I didn't add the AGENTS.md note on this direction because it's outside my write scope. The existing note there already covers it.

`git status` also shows a modified `.gitignore` and a new `docs/design/compact-preview/`. I didn't touch either.
