# Compact preview design QA

The user requested a smaller, simpler factory interface, more run views, model controls, configuration, and file settings. This report covers the local frontend design preview. It does not certify the proposed agent engine.

## Evidence and comparison target

The visual sources are `docs/design/claude-redesign/conversation.png`, `radar.png`, and `reviews.png`. The Sundial refinement and the user's later compactness instructions supersede the older shell geometry.

Each source is 1586 by 992 pixels. Comparison boards normalize the source to 1440 by 900. The browser captures are 1440 by 900 CSS pixels at device scale factor 1. Native narrow captures are 390 by 844 CSS pixels at scale factor 1. No device frame or browser toolbar is included.

The paired evidence is in `docs/design/compact-preview/screenshots/`.

- `compare-conversation.png`, `compare-radar.png`, and `compare-reviews.png` put each source and rendered screen in one image.
- `compare-navigation.png` compares the navigation and typography at a readable scale.
- `compare-review-actions.png` compares the review action bar at a readable scale.
- The seven `*-1440.png` and seven `*-390.png` files have matching `.ax.txt` records of the visible screen state.
- `responsive-measurements.json` records all 35 rendered view and width combinations.

The comparison uses the initial Conversation, active-work Radar, and selected #24 review in Split mode. Runs, Files, and Settings are intentional additions, so they have no older image target.

## Findings and fixes

The first pass was blocked by these issues. The final comparison found no actionable P0, P1, or P2 issues.

| Earlier issue | Fix | Final evidence |
| --- | --- | --- |
| P1. The review footer started at y900 in a 900px viewport. | Place the optional callout, work area, and footer in explicit grid rows. | `reviews-before.png` preserves the failure. `reviews-1440.png` shows the footer within the viewport. |
| P1. The 390px conversation grew to 421px and clipped Send. | Constrain the grid column and its children with minimum widths of zero. Keep the picker label on one line. | `conversation-390.png` shows all controls. The final composer is 366px wide with 12px side margins. |
| P2. Narrow Reviews pushed actions below the comparison area. | Give both Split panels bounded rows and independent code scrolling. | `reviews-390.png` shows the tabs, preview, diff, and actions together. Preview and Diff can each expand through their tabs. |
| P2. The radar caption was clipped at 390px. | Account for the caption's measured height when sizing the dial. | `progress-390.png` and the final caption bounds show the full two-line caption. |
| P2. Keyboard navigation skipped the expanded model controls. | Focus the first select when the panel opens. Escape returns focus to its trigger. | Browser interaction verified both focus targets. |
| P2. An unsupported model and effort combination lacked a linked explanation. | Add a live error, `aria-invalid`, and `aria-describedby`. | Selecting Max on Codex disables Send and announces the recovery choices. |
| P2. File exclusions were missing from new-run context. Retry with current defaults retained old context. | Resolve automatic inputs and explicit attachments into a copied settings snapshot. | Browser submission and retry show excluded AGENTS.md absent while the earlier attempt stays unchanged. State tests cover context capture. |
| P2. A question-mark file pattern could crash rendering. | Translate supported wildcards and escape regex punctuation. | The `???.swift` pattern saves without a crash. State tests cover this pattern and literal punctuation. |
| P2. Refresh lost new runs and could reuse an ID. | Persist local run history, tasks, and messages. Use random IDs with collision checks. | A new run link resolves to the same task after reload. |
| P2. Artifacts trusted a Current label without checking the revision. | Derive stale state when the artifact revision differs. | Source review confirms the same revision rule used by Checks. |

## Required fidelity checks

Typography uses the system sans family, 14px reading text, 13px metadata, and selected 12px compact labels. The source's larger text and icons are intentionally smaller. Monospace identifies code and run IDs. Reading text remains 14px at narrow widths.

The shell measures 168px of navigation and a 40px header on desktop. The 208px review queue leaves 1064px for the selected review at 1440px, about 84 percent of the content width. The work area uses compact seams and 16px desktop gutters. Navigation becomes a modal drawer at 768px. The review queue becomes a selector at 1024px.

The warm paper palette matches the selected direction. The checker passes 85 pairs using colors read from the rendered page. The lowest ordinary text pair is 4.63 to 1. Required boundary pairs reach at least 3.22 to 1. Decorative rings and separators have a separate role. The result is in `contrast-results.json`.

The preview uses the existing Green Room collection screenshot. It preserves the photograph and record artwork without substitution or stretching. The source screenshot hash is recorded in the earlier design provenance. Radar is a data visualization drawn with Canvas. Phosphor supplies the interface icons. The source's generated macOS traffic lights are absent because this is a browser preview.

Copy retains one human-facing Factory lead, one project objective, and short progress summaries. Reviews owns changes and review actions. Radar contains active Plan, Build, Check, and Waiting work, with stage age controlling radius. Sample states and local actions have explicit preview labels. No hidden reasoning transcript appears.

## Runtime verification

The production build passes with Node 24.22.1 and pnpm 11.25.0. All four state tests pass. The protected template runtime files retain their original hashes.

Browser checks cover Conversation, Runs, Run detail, Radar, Reviews, Files, and Settings at widths 1440, 1280, 1024, 768, and 390. The final 35 cases have no page-width overflow, clipped composer, offscreen review footer, or clipped radar caption. Code and wide tables scroll inside their own regions.

The browser interaction pass verified the following behavior.

- Native model selection, an unavailable catalog option, unsupported effort recovery, keyboard focus, and Escape return.
- Project defaults, a retained next-run override, submission snapshots, file exclusions, and local history after refresh.
- Shared List and Timeline search, return navigation, arrow-key tabs, retry with current defaults, and sibling attempt History.
- Pending interruption followed by Interrupted without a new attempt.
- Stale checks tied to an older revision.
- Local file edits and saved content after refresh.
- Review dialog Escape and focus return, a simulated merge, and reset to clean sample data.

The final browser passes produced no new console warnings or errors. The session retains one earlier Vite hot-reload error from assembly while page imports were still being written. The restarted server and final production build load the complete source.

Prove It Works guided the verification. The review used the running interface, actual DOM geometry, state-paired captures, and an independent visual comparison. Compilation alone was insufficient.

## Remaining limits and polish

The model catalog and execution records are examples. Settings persist in this browser. There is no Rust supervisor, TUI, native Tauri shell, live provider connection, deployment monitor, or GitHub merge. Steering delivery and reconnect states from the larger interaction contract remain outside this design pass.

Full screen-reader testing, browser-matrix testing, and text zoom beyond the tested responsive widths were not performed. Future production work must test those with the actual engine and native client.

P3 polish remains in wrapped review titles, native select styling, and monochrome code. The queue width and native controls are deliberate compactness choices. Dense radar labels still need a collision strategy when real work can contain many simultaneous tasks.

## Implementation checklist

- Compact shell and separate Reviews are implemented.
- Run views and scoped settings controls are connected.
- File context and sample editor choices work locally.
- Blocking visual and interaction findings are fixed.
- Final source comparisons and responsive captures are saved.
- The preview remains a design decision artifact. Production behavior requires the implementation milestones in `docs/getting-started.md`.

final result: passed

## One-off refinement — November 1, 2026

This follow-up adds one-off request intent to the same lead conversation and a third Runs view, Schedule. The earlier findings and 35-case report above describe the preceding compact preview. Message submission now saves a local conversation entry rather than creating an execution attempt. This section supersedes earlier new-run submission descriptions for the composer; retry still creates a local example attempt.

### Source and rendered evidence

The accepted compact implementation and the user's latest compactness and one-off instructions are the comparison target. Claude through Cursor CLI supplied a bounded plan-mode consultation, recorded in `docs/design/compact-preview/oneoff-claude-brief.md` and `oneoff-provenance.json`. Native collaborators and the parent implemented and reviewed this feature; the consultation was not a live factory run.

`docs/design/compact-preview/oneoff-screenshots/compare-oneoff-composer.png` places the preceding compact Conversation capture beside the current One-off entry at 1440 by 900. `compare-mobile-flows.png` shows the five new states at their native 390 by 844 size. Schedule is an intentional addition with no older raster target. The warm-paper palette, system type, compact navigation, and separate Reviews and Sundial surfaces remain the accepted design language. New controls inherit the checked semantic color pairs; no palette tokens changed in this pass.

The evidence directory contains paired screenshots and AX text for collapsed and expanded scheduled composers, Schedule list, selected request, and rescheduling dialog at widths 1440, 1280, 1024, 768, and 390. Desktop/tablet captures are 900px high; phone captures are 844px high. Device scale is 1. The final `responsive-measurements.json` records all 25 cases, with no page overflow or offscreen measured controls. Four stale composer captures were detected by checking bitmap dimensions and recaptured after the viewport settled; all final bitmap sizes match their recorded viewport.

### Findings resolved

| Finding | Final behavior and evidence |
| --- | --- |
| P2. Scheduled labels omitted the year. | All scheduled policy labels include the year and offset; date tests distinguish 2027 and 2028. |
| P2. Closing a cancelled request under Waiting lost keyboard focus. | Back focuses the original row when present and the Schedule heading otherwise. The actual cancel/Waiting/Back journey lands on the heading. |
| P2. Mobile table metadata resisted its hide rule and lengthened every row. | Corrected selector specificity. Phone rows show request, When, and Status; model/context remain available in selected details. |
| P2. Mobile Schedule controls occupied four rows, and detail retained irrelevant list controls. | Phone toolbar uses three compact rows. Selected detail takes the full work area below 1180px, with a clear Back action. |
| P3. A successful send removed focus with its now-disabled Send button. | Focus returns to the composer after submission, including keyboard submission. |

Independent final source review found no remaining P1 or P2 findings. Parent visual review found no actionable P0, P1, or P2 issue in this bounded feature.

### Interaction and state verification

Browser QA used the separate `localhost` origin so it did not reset or populate the user's `127.0.0.1` data. Temporary QA tabs were closed. The user's preview was left in One-off mode with its existing conversation and settings preserved.

- Lead decides, Next available, and a future Schedule at request submit through the one lead conversation. Empty scheduled time is rejected; a valid timestamp shows its resolved offset and IANA zone. A successful send resets mode, timing, attachments, and overrides.
- The receipt opens the matching Schedule detail. Requests and a one-off draft survive reload with stable identities. A new one-off opens the existing composer and focuses it.
- After changing project defaults from Codex/Medium to Claude/Max, an earlier request still shows Codex/Medium and its source at send. Rescheduling changes timing while preserving text, identity, requested settings, and context.
- Cancellation leaves the record inspectable, disables rescheduling and repeated cancellation, and preserves execution history. Search, empty filter results, and Clear filters work.
- The unsupported Codex/Max combination disables Send with a linked live explanation. Recovery to High succeeds; Escape returns focus from the model panel. Reschedule dialog Escape returns focus to Reschedule. Arrow keys switch Timeline and Schedule correctly. Cmd+Enter submits a valid one-off and returns focus to the composer.
- Sending four one-off requests and one message leaves the nine sample execution attempts unchanged. Graph retains its sample tasks; Radar retains only the three active/waiting sample marks. Nothing dispatches on a timer.
- The final browser console contains no warnings or errors.

The production build and all ten state/date tests pass with the bundled Node 24 and pnpm 11. Tests cover future-time rules, real Los Angeles DST gaps/overlaps, distinct scheduled years, copied snapshots, reschedule/cancel identity, derived Due status, malformed restore records, context exclusions, and saved attempt history. The four protected template runtime hashes are unchanged. `git diff --check` passes.

### Limits

This is still a browser-local request preview. There is no supervisor dispatch, durable engine queue, provider call, calendar wakeup, or execution guarantee at a requested time. Due-state transition and malformed/storage restore behavior are covered at the state boundary; browser storage-quota failure was not fault-injected. Full screen-reader, browser-matrix, and zoom testing remain outside this pass. Real orchestration belongs to the planned runtime milestones.

Prove It Works used the running UI, real saved requests, source review, native paired captures, measured bounds, and focused state tests. No additional broad test suite was added.

final result: passed

## Conversation polish — November 1, 2026

The user asked for a modest UI improvement. This pass changes only presentation and initial conversation scroll positioning in `src/App.jsx` and `src/styles.css`: intent/timing before the draft, a quieter next-send control, grouped consecutive Factory messages, conversation content anchored near the composer, clearer approval/review handoffs, and consistent selection markers. The input boundary stays visible; ordinary text sizes and the warm-paper palette are preserved.

Claude Opus 5.5 supplied a scoped design plan through Cursor CLI after inspecting the running preview's saved screenshot and source. The public plan and parent decisions are in `docs/design/compact-preview/polish-claude-brief.md`; session details are in `polish-provenance.json`. A separate source reviewer found no actionable P1/P2 regression. The parent retained the strong composer boundary, preserved the single request status row, and kept non-Ask modes visible as their own label when the model name truncates.

The evidence directory is `docs/design/compact-preview/polish-screenshots/`. `compare-conversation.png` compares the actual before and final implementation at the same 1341×1224 viewport. The final clean Conversation captures are `conversation-1440.png` and `conversation-390.png`, with paired AX records. `plan-only-390.png` confirms the final exceptional-mode label is visible. The dated and expanded screenshots document the layout iteration; they precede that last label separation. Review captures show the destination at 1440 and 390px.

The scheduled and expanded composer states pass ten measured combinations at widths 1440, 1280, 1024, 768, and 390, with no page overflow or offscreen form controls. A final 390px check verifies the separate Plan only label and Send remain within the composer. The full review handoff title wraps at 390px without clipping. The selected review footer stays in view at both desktop and phone widths. Shared Files, Reviews, navigation, and Settings selection markers are present in the rendered page. Six new color usages pass contrast checks; the lowest marker pair is 4.77:1, and ordinary text pairs exceed 4.5:1. No semantic palette value changed.

Browser checks verified mode changes preserve the draft, tab order reaches When then the draft, the model panel receives focus and returns it on Escape, invalid Codex/Max still disables submission, and recovered settings retain Plan only in both the visible control and accessible name. Drafts and overrides survive reload. A long restored conversation opens at its latest messages and can scroll back to the first message. Cmd+Enter submits a locally saved one-off, resets to Message, and returns focus to the draft. The nine sample execution attempts remain unchanged. Local approval feedback and the review handoff still work. No browser warnings or errors were recorded.

QA writes stayed in the separate localhost test origin; the user's 127.0.0.1 conversation, drafts, defaults, and requests were preserved. The final production build passes, all ten existing state/date tests pass, and protected template runtime hashes are unchanged. No styling-value tests or dependencies were added.

This is UI polish for the local preview. Live orchestration, full screen-reader/browser-matrix testing, and text zoom remain outside this pass. Long model names deliberately truncate in narrow controls; expanding the native settings panel exposes full values.

final result: passed

## Bottom composer, permissions, and Scratchpad — November 1, 2026

The user requested a quieter Message / One-off control at the bottom, explicit permissions, and a note Scratchpad reviewed every morning on a configurable cadence. The accepted compact frontend and those instructions are the comparison target. Claude Opus 5.5 supplied a bounded plan through Cursor CLI; the public brief and execution provenance are in `docs/design/compact-preview/scratchpad-claude-brief.md` and `scratchpad-provenance.json`. A native implementation owner, independent source reviewer, and the parent completed this refinement. This consultation and local triage are not live factory execution.

### Rendered evidence

Evidence is in `docs/design/compact-preview/scratchpad-screenshots/`. The bottom composer, permissions dialog, Scratchpad, and expanded review configuration were measured at 1440, 1280, 1024, 768, and 390 CSS px: 20 combinations. Desktop/tablet heights are 900px; phone height is 844px, with device scale 1. The final measurements record no page overflow, no horizontal control overflow, and no clipped controls in the fixed composer/dialog. Scratchpad uses its normal vertical scroll region for longer notes and configuration. The permissions dialog initially placed Use project defaults beyond the phone's left edge; the final footer wraps that action onto its own row and all five dialog widths pass.

Paired PNG and AX captures include desktop/phone Scratchpad, expanded desktop configuration, desktop/phone permissions, and the phone composer. The phone composer shows Plan only separately from the truncated model name and puts Type / Send on its bottom row. `permissions-before-390.png` preserves the resolved finding; `permissions-390.png` is final. Final captures of the user's unchanged original-origin data are `scratchpad-user-config-1280.png` and `bottom-composer-user-1280.png`, both at 1280×720. Other captures use isolated QA data. All saved PNG dimensions were checked against their named viewport. Parent visual inspection found no remaining actionable P0/P1/P2 issue in these surfaces. Semantic color tokens and ordinary type sizes remain unchanged; prior contrast results apply to the reused pairs.

### Interaction and state verification

QA mutations used the separate localhost origin. The user's 127.0.0.1 notes, drafts, defaults, and requests were preserved. A temporary read-only original-origin tab captured the final screens and was closed; the user's Reviews tab and running server remain available.

- Message type changes preserve the draft. The native selector, model, Permissions, and Send stay at the composer bottom. The permissions dialog closes with focus returned to its trigger. Broader next-send permissions are disabled when the project ceiling is narrower.
- Scratchpad starts at daily 09:00 in the browser IANA zone. Drafts, notes, edited revisions, archives/restores, and valid review-plan changes survive reload. Invalid IANA zones leave the last saved plan intact. Daily time, hourly, every 15 minutes, and pause intent save correctly. Cron is derived in expanded configuration; the disconnected-runtime explanation is visible.
- Unsaved note edits disable triage and archive. A blank Cmd/Ctrl+Enter save retains the editor and draft. A valid explicit save increments revision. Archive/restore preserves the record; sent notes remain immutable.
- Preview triage atomically creates one lead-policy Request per inbox note, with stable note/revision provenance, then marks notes sent. It captures project settings/context rather than unsent composer overrides. No Run, task, provider acknowledgment, or Radar mark is created. Linked Request inspection and exact source-note return select the proper note/filter and move keyboard focus to it.
- Cancelling a generated Request preserves its source and sent note. Reload and repeated triage cannot duplicate it. Request identity and captured permissions remain intact after project defaults change.
- Ordinary Message also captures a copied requested settings/context snapshot, available under Requested settings. New one-offs, triaged notes, and retries explicitly based on current defaults capture resolved permissions. Older runs show Not captured; they do not inherit new defaults. A deliberate retry from current defaults creates one local example attempt with those current values.
- Final QA console inspection returned no warnings or errors.

### Review and checks

Independent review identified two P2 issues: blank keyboard saves discarded the edit and hourly configuration validated an inapplicable daily-time field. The parent fixed both and verified them in the browser. It also added ordinary Message capture and exact source-note navigation. Final independent source review found no actionable P1/P2 finding in the connected fixes.

The final production build passes with bundled Node 24 and pnpm 11. All 19 focused state, one-off, permissions, and Scratchpad tests pass. Added tests cover restrictive permission resolution, legacy Not captured evidence, snapshot copying, default/configurable cron, zone validation, note lifecycle, atomic/idempotent promotion, cancelled-origin retention, unsaved/invalid inputs, and malformed restoration. No dependencies or styling-value tests were added. The four protected template runtime hashes are unchanged; `git diff --check` passes.

### Runtime boundary

This preview saves review configuration and manually promotes note text. No background cron, semantic review agent, durable supervisor, lead scheduling, provider call, or permission enforcement is connected. A future runtime must enforce capabilities, reconcile source revisions, and define missed-slot recovery before dispatch. Storage-quota failure was not fault-injected. Full screen-reader, browser-matrix, and text-zoom testing remain outside this pass.

Prove It Works used the running UI, persisted state journeys, paired native captures, measured bounds, focused domain tests, and an independent connected-code review.

final result: passed

## Blank-document Scratchpad — November 1, 2026

The latest user direction supersedes the note-list layout above: Scratchpad is one blank document with agent-completed passages highlighted, and review settings belong in Settings. The implemented surface has a native autosaved textarea, a quiet title, and local-save status. It has no capture button, note cards, lifecycle filters, or schedule controls. Settings → Scratchpad owns the existing morning plan and manual preview action. The public Claude consultation and parent decisions are in `docs/design/compact-preview/document-claude-brief.md` and `document-provenance.json`.

### Source identity and editing

Existing visible notes, unfinished edits, and the capture draft migrate once into canonical document text; legacy records remain for Request provenance. A saved empty document remains empty after reload. Blank lines separate new preview passages. Triage processes only unconsumed document ranges and captures project settings/context. Editing text never rewrites an earlier Request.

Green marks require exact note/revision/text provenance, a matching Request, and its latest linked execution attempt succeeding. Pending, scheduled, cancelled, failed, unrelated, and superseded execution evidence cannot mark a passage done. Completion is also exposed as an accessible count and expandable links to matching Runs. No real completion is fabricated in the preview.

The independent review found two P2 issues during implementation: repeated text could inherit the wrong anchor after deletion, and native Undo could restore text without its submitted identity. Both are fixed. Native beforeinput intervals preserve exact occurrence identity; missing/invalid intervals invalidate ambiguous ranges conservatively. Document-only session snapshots handle native Undo/Redo, including grouped edits, while preserving current configuration and immutable Requests. The final independent review reports no remaining P1/P2 finding.

### Browser and visual evidence

Evidence is in `docs/design/compact-preview/document-screenshots/`. Fifteen measured view/width combinations cover the full-app document, relocated Settings, and the completion component at 1440, 1280, 1024, 768, and 390 CSS px. Desktop/tablet height is 900px; phone height is 844px. There is no page overflow or horizontal control overflow. The document and its save/completion footer stay within the viewport; Settings uses normal vertical scrolling. The phone Settings categories now wrap so the selected Scratchpad category is visible after reload; the earlier hidden-selection capture is preserved as `settings-before-390.png`.

Paired screenshots and AX records include `document-390.png`, desktop/phone Settings, desktop/phone completion examples, and wrapped/scrolled long text. The completed-range mirror and native textarea have matching measured boxes and computed fonts at all five widths. Long-phone scrolling produced matching 1688px scroll offsets and matching 4856px content heights. Native editing retains opaque textarea glyphs; the aria-hidden mirror supplies only mark backgrounds. Parent screenshot inspection found no remaining actionable P0/P1/P2 issue in this bounded change.

The completion examples used a temporary component harness with explicit succeeded/running linked fixtures and a visible QA label. It wrote no localStorage. Source and harness files were removed before the final build. That temporary harness logged createRoot warnings during source hot reload; the shipped app's final console check contains no warnings or errors. Example evidence is not a live agent run.

The final original-origin capture, `document-user-1127.png`, is at the user's normal 1127×1224 viewport. The user's original data remains intact and the original tab is left on Scratchpad. All QA writes occurred in the separate localhost origin; temporary tabs are closed and responsive overrides are reset.

### Verified journeys and checks

- Migrated text and a newly added multiline passage survive navigation/reload. A deliberately cleared document stays blank despite preserved legacy notes.
- Settings validates an invalid IANA zone without replacing the saved plan, saves hourly/UTC, and restores daily 09:00/Los Angeles. The selected Settings category survives reload through its route.
- Preview triage creates one Request for the newly added passage, without creating a Run or completion mark. Its Schedule source link selects the exact passage and focuses the document.
- Native appended text ending with repeated punctuation preserves existing consumed ranges. Deleting the first of two identical paragraphs leaves the unfinished second passage consumed and unmarked. Native Undo restores both source identities and one completed mark; Redo again retains only the unfinished second source, with zero new pending passages throughout.
- Completion details expose the matching Run link. Long text wraps and scrolls in the same document surface; pending text remains unhighlighted.

The final production build and all 32 focused tests pass. Thirteen new document tests cover migration, malformed/stale ranges, precise and ambiguous edits, native history, pending ranges, atomic/idempotent triage, and latest-attempt completion identity. The existing 19 tests still pass. No dependency was added. Four protected template runtime hashes are unchanged; local documentation links, PNG dimensions/sidecars, recorded geometry, and `git diff --check` pass.

The preview still has no live cron, agent triage, lead scheduler, provider dispatch, or permission enforcement. Full IME, screen-reader, browser-matrix, and text-zoom testing remain outside this pass. Native text Undo/Redo and source identity were tested in the available browser; history snapshots are local to the mounted editor session.

final result: passed

## Reviews overhaul — November 1, 2026

The user requested a total overhaul of Reviews. The accepted compact shell and current rendered Reviews capture are the visual source; the new flow intentionally replaces the persistent queue and default Split layout. A compact inbox opens each change in a full-width desk, with Preview, Changes, and Evidence modes. The existing screenshot is shown at a useful 520px desktop width with vertical scrolling; native-pixel zoom pans inside its own surface. No before image, interactive demo, or live preview was invented.

Claude Opus 5.5 supplied a read-only design plan through Cursor CLI. The public recommendations, parent decisions, and provenance are in `docs/design/compact-preview/reviews-overhaul-brief.md` and `reviews-overhaul-provenance.json`. Native agents owned the UI, evidence/state helpers, and an independent connected-code review; the parent integrated, reviewed, and drove the running UI. The consultant's six-column inbox and device-point zoom ideas were simplified to the current compact rows and explicit native-pixel zoom.

### Findings and fixes

- P2. Existing checks could look passed based on a Ready label, even when evidence was stale or absent. Reviews now pin run, attempt, and revision; exact-revision checks alone count. Stale results have explicit not-counted labels and no green pass styling.
- P2. Missing source records could crash, and merge could be simulated despite invalid evidence. Missing/changed source and newer task attempts now have visible blocking reasons. Merge requires the latest succeeded source and nonempty current passing checks. Confirmation rechecks its opening source snapshot.
- P2. Old review IDs were ignored and feedback disappeared on close/navigation. Stable review routes now resolve the exact review, unknown IDs recover without selecting an unrelated item, and per-review drafts and local decision snapshots persist in the existing aggregate.
- P2. A successful merge removes the trigger that a closing dialog normally refocuses. Focus now moves to the stable review heading. Cancel/Escape restore the trigger. Back restores the inbox row or heading when that row is filtered out.
- Resolved polish: programmatic heading focus no longer draws a text-input-like box, feedback starts in its writing field, Edit request preloads recorded feedback, storage failures have readable copy, and keyboard focus outlines stay inside clipped preview/code scrollers.

Final independent source review found no remaining P1/P2 issue. Parent visual review found no actionable P0/P1/P2 issue after these fixes.

### Visual comparison

Evidence is in `docs/design/compact-preview/reviews-screenshots/`. `before-1440.png` and `preview-1440.png` show the same review #24, source `a4f3c2e`, Ready status, project, theme, 1440×900 CSS viewport, and device scale 1. Source mode is Split and final mode is Preview by design. `compare-review-desk.png` places both complete captures together; `compare-review-controls.png` compares their headers and decision bars at native scale. The boards were opened alongside native phone Changes and Evidence captures. Geometry changes are the requested overhaul, rather than a fidelity mismatch with the old queue/split design.

Required surfaces were checked explicitly. Typography retains the system sans/mono families, readable 14px body and 13px metadata, with a stronger 17–18px workspace title and 16px phone title. The new 24px desktop gutters, single mode seam, full-width stage, and pinned decision bar establish hierarchy without nested cards. Existing warm-paper, sage, warning, danger, and diff tokens are reused; no palette value or font dependency changed. The original 1206×2622 PNG retains its aspect ratio and is not stretched or redrawn. Phosphor icons match the existing shell. Copy names the source, missing/older artifacts, feedback scope, and local-only actions rather than claiming external execution. The earlier artifact gaps are now surfaced together in Evidence, so its note count is broader than the old single limitation link.

Desktop/phone pairs cover inbox, Preview, Changes, Evidence, and feedback. Twenty-five nonempty measured state/width cases pass at 1440, 1280, 1024, 768, and 390 CSS px, with 900px desktop/tablet height and 844px phone height. Mode/dialog state was reapplied at each viewport before measurement. No page overflow, offscreen primary control, or clipped decision footer remains. Code and evidence tables deliberately scroll inside their own surfaces. At phone width, native zoom measured a 1206px image inside a 375px visible scroller while the page stayed 390px wide.

The final user-window capture is `preview-user-1445.png`, at the normal 1445×1224 viewport. Responsive overrides were reset; the user's tab remains on review #24. All screenshots have paired AX records and checked bitmap dimensions. The two comparison boards are diagnostic compositions of saved captures.

### Verified interactions and state

- Inbox search/status filters, empty-result recovery, historical-merge explanation, and exact review links work. Back preserves query/filter and returns focus to the selected row. Previous/Next retains each review's content mode and chosen file. Arrow keys switch modes. Conversation and Run handoffs open the exact review; its Run link opens the source attempt.
- Preview switches between useful-width and native pixels. Changes uses a full-path file selector and contained code scroll. A review without a screenshot opens in Changes; its Preview mode explains the gap and links back to Changes.
- Feedback remains attached to its own review across Cancel, other reviews, navigation, and reload. Escape restores trigger focus; opening feedback focuses its textarea. Recording a change request preserves its source and history, blocks merge while changes are outstanding, and reloads the same state.
- A merge was simulated only on the separate localhost QA origin. Its outcome and source snapshot survive reload, repeated actions are unavailable, and keyboard focus returns to the heading. No Run was created or rewritten, and no GitHub call occurred. The original 127.0.0.1 state was used only for read-only view captures.
- A temporary memory-only component harness verified stale checks, missing source, newer attempts, and a source revision changing while confirmation stayed open. Stale evidence produced six not-counted labels and zero green checks. Missing source disabled both decisions. A newer attempt blocked merge. Confirmation after the timed source change showed an error and left Ready unchanged. These explicit QA fixtures were not factory work; their files and tabs were removed before the final build.
- Final app and fixture console checks returned no warnings or errors.

### Checks and limits

The final production build passes. All 41 focused state/one-off/permission/document/review tests pass, including nine review tests for exact evidence, missing optional previews, malformed/missing source, latest-attempt eligibility, changed-source confirmation, immutable decisions, duplicate actions, and independent restored drafts. No dependency was added. Four protected template runtime hashes are unchanged, documentation links resolve, recorded bounds and PNG/AX pairs pass, and `git diff --check` passes.

Real GitHub reviews/merges, live application demos, external check refresh, provider calls, and the Rust supervisor remain unconnected. The UI was tested in the available browser; full screen-reader, browser-matrix, and text-zoom testing were not performed. Storage-quota failure was not fault-injected.

Implementation checklist: inbox and desk routes connected; full-width modes implemented; evidence and decisions source-bound; feedback and local outcomes persisted; focus and responsive checks passed; temporary harness removed; user data preserved.

final result: passed

## Composer timing placement — November 1, 2026

The user's annotated When selector and help now sit in the bottom action row with One-off and Send, below the model and Permissions row. The default help stays inline on desktop; scheduling details wrap within that bottom section. The writing area remains clear. Existing handlers, validation, description IDs, and draft state are preserved; Send follows timing in keyboard order.

Browser verification covered Lead decides, Next available, Schedule at, and Message at 1440, 1280, 1024, 768, and 390 CSS px. All 20 cases have no page overflow; timing stays below metadata, Send remains contained, and Message has no timing controls. A native keyboard date edit produced the expected past-date error and aria-invalid state. Tab from Lead decides reaches Message type. The build and git diff whitespace check pass, and the original tab has no console warnings or errors. No new automated test was added for this layout change; provider dispatch remains outside this preview.

Before/after screenshots, the phone scheduled-date view, past-date feedback, paired accessibility snapshots, and geometry measurements are saved in `../../docs/design/compact-preview/composer-screenshots/`. Draft and policy checks used the separate localhost QA origin; temporary tabs were closed and the viewport reset. The user's 127.0.0.1 draft was not edited.

final result: passed

## Conversation flow and project permissions — November 1, 2026

Routine goal/assignment copy now collapses behind a native disclosure, leaving approvals and review outcomes visible. Own sends have one receipt beneath the message instead of a duplicate success notice above the composer. The conversation follows incoming updates when already near the bottom, preserves reading position otherwise, and offers Latest updates. Own sends follow the new message and return composer focus. Smooth scrolling has an explicit reduced-motion branch.

The Permissions dialog separates saved project rules from next-send restrictions and links to Settings → Permissions. That editor names the project, describes edit/command/network/integration scope, and saves with Save project permissions. Direct entry focuses its selected tab. Project ceilings, default Ask values, and immutable capture logic are unchanged.

Executed browser checks on a separate localhost QA origin: saved Read only + Network Deny survived reload; the composer inherited those rules and disabled broader options. A pre-change send retained Worktree edits + Network Allow while a later send captured Read only + Network Deny. Both snapshots remained inspectable, and the success Notice stayed empty with one receipt per send. The QA project's previous rules were restored afterward. The original 127.0.0.1 settings and draft were not edited. Escape from the dialog restored Permissions trigger focus.

A temporary memory-only Conversation fixture exercised real copied component logic with 48 clearly labelled QA messages. Initial and near-bottom incoming-update gaps were 0px. While reading 1440px above the end, a new update kept scrollTop exactly 2181px and exposed Latest updates. That button returned to a 0px gap. A final keyboard check confirmed Enter on Latest updates returns composer focus before hiding the button and ends at a 0px gap. An own send from older history ended at a 0px gap with a blank draft and composer focus. The fixture reported no console warnings/errors and created no factory work. Its files and tabs were removed before the final build.

All 15 rendered layout cases (Conversation, project permissions, and dialog at 1440/1280/1024/768/390px) had contained widths and no page overflow. Final native user-origin phone/desktop screenshots and AX pairs, geometry data, and the explicit scrolling assertions are in `../../docs/design/compact-preview/agent-flow-screenshots/`. The original tab was restored to its normal viewport. All 41 focused tests pass, including all three permission tests. The production build and whitespace check pass. Full screen-reader, reduced-motion emulation, live provider activity, and OS permission enforcement were not tested; the runtime remains disconnected.

final result: passed
