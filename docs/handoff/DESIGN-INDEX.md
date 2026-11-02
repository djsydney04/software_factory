# Design files and approved implementation

The runnable frontend and latest [desktop instructions](../../apps/desktop/AGENTS.md) are the current product. Older raster mocks are useful context, not a reason to restore crowded layouts. Read [the current design contract](../design/compact-preview/README.md) and its linked interaction contracts before changing a surface.

## Current screen references

These are captured app views, not claims that sample tasks or providers ran. Desktop filenames sometimes describe the intended capture width rather than the final native viewport; geometry/AX records and the QA log contain measured dimensions.

| Surface | Desktop reference | Phone / related reference | Implementation |
| --- | --- | --- | --- |
| Conversation: quiet updates, bottom composer, borderless timing | [Latest conversation](../design/compact-preview/agent-flow-screenshots/conversation-user-desktop.png) | [390px](../design/compact-preview/agent-flow-screenshots/conversation-user-390.png) | `apps/desktop/src/App.jsx`, `styles.css` |
| One-off timing placement and treatment | [Borderless When control](../design/compact-preview/composer-screenshots/oneoff-borderless.png) | [Scheduled date at 390px](../design/compact-preview/composer-screenshots/oneoff-scheduled-390.png) | Same composer; later agent-flow captures own the activity and permission refinements |
| Project permission editor | [Latest settings](../design/compact-preview/agent-flow-screenshots/project-permissions-user-desktop.png) | [390px](../design/compact-preview/agent-flow-screenshots/project-permissions-user-390.png) | `App.jsx`, `Permissions.jsx`, `permissions.js` |
| Composer permissions and inherited project rules | [Dialog](../design/compact-preview/agent-flow-screenshots/permissions-dialog.png) | [390px](../design/compact-preview/agent-flow-screenshots/permissions-dialog-user-390.png) | `App.jsx`, `Permissions.jsx` |
| Blank-document Scratchpad | [Document](../design/compact-preview/document-screenshots/document-user-1127.png) | [390px](../design/compact-preview/document-screenshots/document-390.png) | `Scratchpad.jsx`, `scratchpad.js` |
| Configurable morning review plan, in Settings | [Scratchpad settings](../design/compact-preview/document-screenshots/settings-1440.png) | [390px](../design/compact-preview/document-screenshots/settings-390.png) | `Scratchpad.jsx` |
| Runs and selected attempt | [List](../design/compact-preview/screenshots/runs-1440.png), [detail](../design/compact-preview/screenshots/rundetail-1440.png) | [Timeline](../design/compact-preview/screenshots/timeline-1440.png), [390px detail](../design/compact-preview/screenshots/rundetail-390.png) | `Runs.jsx`, `data.js` |
| One-off requests in Schedule | [List](../design/compact-preview/oneoff-screenshots/schedule-list-1440.png), [detail](../design/compact-preview/oneoff-screenshots/schedule-detail-1440.png) | [390px](../design/compact-preview/oneoff-screenshots/schedule-detail-390.png) | `Schedule.jsx`, `scheduling.js` |
| Progress: Sundial active stages | [Radar](../design/compact-preview/screenshots/progress-1440.png) | [390px](../design/compact-preview/screenshots/progress-390.png) | `Progress.jsx` |
| Progress: graph and board | [Graph](../design/compact-preview/screenshots/graph-1440.png), [Board](../design/compact-preview/screenshots/board-1440.png) | Same Task/Run records as Radar | `Progress.jsx` |
| Reviews inbox | [Inbox](../design/compact-preview/reviews-screenshots/inbox-1440.png) | [390px](../design/compact-preview/reviews-screenshots/inbox-390.png) | `Reviews.jsx`, `reviews.css` |
| Large selected review desk | [Preview](../design/compact-preview/reviews-screenshots/preview-user-1445.png), [Changes](../design/compact-preview/reviews-screenshots/changes-1440.png), [Evidence](../design/compact-preview/reviews-screenshots/evidence-1440.png) | [390px Preview](../design/compact-preview/reviews-screenshots/preview-390.png), [feedback](../design/compact-preview/reviews-screenshots/feedback-dialog-390.png) | `Reviews.jsx`, `review-state.js`, `reviews.css` |
| Files and context | [Files](../design/compact-preview/screenshots/files-1440.png) | [390px](../design/compact-preview/screenshots/files-390.png) | `App.jsx`, `data.js` |

`data.js` contains seeded records and illustrative model choices. It is not a live capability registry. Current settings and file controls also live in `App.jsx`; do not create a second frontend to connect the runtime.

## Product decisions to retain

- One human-facing lead conversation, with worker detail in Runs/Progress. Quiet summaries, useful approvals, one send receipt, and scrolling that preserves reading position.
- Warm paper, sage/amber accents, clear hierarchy, 168px rail, 40px header, 14px body and 13px metadata. Compactness comes from removing duplication rather than making text unreadable.
- Model, permission, Message/One-off and timing controls at the bottom. The timing select has a transparent border/background. Scope must distinguish project defaults, next-send restrictions and captured history.
- Scratchpad is a continuous autosaved document. Scheduling belongs in Settings. The saved plan defaults to daily 09:00 in the browser's IANA zone; a future runtime must retain that intent and source-bound completion marks.
- Sundial is our own stage/age design. Use [its refinement](../design/claude-redesign/radar-refinement.md) for meaning, with current compact shell geometry. No changes, PRs, diffs, previews or merge controls in Radar.
- Reviews uses inbox → full-width desk, stable review IDs, separate Preview/Changes/Evidence modes, and large inspection space. No permanent queue or Split view. Stale checks cannot certify a newer revision.
- Verify desktop and phone widths: 1440, 1280, 1024, 768 and 390 CSS px. Preserve native keyboard behavior, drafts, selected object identity and explicit disconnected states.

## Every generated raster mock, retained as history

All nine are included in the ZIP. None overrides the later source/interaction decisions above.

| Exploration | Files | What supersedes it |
| --- | --- | --- |
| Initial three screens | [Conversation](../design/conversation.png), [Graph](../design/progress-graph.png), [Radar](../design/progress-radar.png) | Current compact hierarchy and single lead workflow |
| First compact exploration | [Conversation](../design/compact-conversation.png), [Graph](../design/compact-graph.png), [Radar](../design/compact-radar.png) | Current source; these older views mix review information into monitoring |
| Claude redesign raster sources | [Conversation](../design/claude-redesign/conversation.png), [Radar](../design/claude-redesign/radar.png), [Reviews](../design/claude-redesign/reviews.png) | Current compact geometry, refined Sundial, blank Scratchpad, bottom composer and Reviews inbox/desk |

The [Claude design brief/render prompts](../design/claude-redesign/README.md), [pre-Claude notes](../design/pre-claude-notes.md), and [archived public consultations](archive/README.md) explain the design history. Some historical prompts describe queued demo runs, note cards or queue/Split Reviews. Do not follow those superseded instructions as the next task.

## Original references and example imagery

The [original user request](source/goal-objective.md) has only its five attachment paths made relative for portability. The [five original reference PNGs](source/README.md) are copied byte-for-byte with hashes recorded. Use their visual principles rather than copying product identities or assuming their features exist here.

The review example uses [`apps/desktop/public/assets/greenroom-collection.png`](../../apps/desktop/public/assets/greenroom-collection.png). It is sample review imagery, not a captured result from a real Factory PR or deployment. Keep this fixture out of live evidence.

## Evidence and complete asset inventory

All `docs/design/` assets remain included: tokens, layout budgets, contrast records, provenance, image prompts, current/previous screenshots, and accessibility snapshots. The [snapshot manifest](manifest.json) enumerates every packaged path and its hash. The [QA record](../../apps/desktop/design-qa.md) explains executed checks and limits; it has separate sections for each refinement.

Screenshots named `before`, `initial`, `compare`, or `fixture` show historical states, diagnostics or deliberately labelled QA conditions. A fixture proving stale checks or completion highlighting is evidence about the UI test, not actual factory work. The `agent-flow-screenshots`, `document-screenshots`, and `reviews-screenshots` references above identify the latest relevant surfaces; earlier note-based Scratchpad and composer-placement captures remain history.
