# One-off interaction refinement

Claude Opus through Cursor CLI. The implementation follows these compact layout recommendations, with request states kept separate from execution attempts.

I'd keep this shape. Below are the exact labels and density changes, then the two main UX risks.

**Composer (Conversation)**
- Add a two-option toggle at the left of the composer footer: `Message | One-off`. Keep the existing `Next run Codex GPT-5.3 · Medium · Ask first` chip.
- When One-off is selected, put one native select in the same row: `When: Lead decides`, with options `Lead decides`, `Next available` and `Schedule at…`.
- `Schedule at…` shows an inline local date-and-time field plus the time zone, e.g. `PST`. It stays in the same row, so the composer doesn't grow taller.
- One-off placeholder: "Describe one task for the lead to schedule". Send tooltip: `Queue one-off`.
- After sending, the user bubble gets an 11px meta line: `One-off · Lead decides · Codex GPT-5.3 · Medium`. Below it, a quiet system row says `Request saved · not started` with a link `Open in Schedule →`. Don't show a Factory reply, because nothing has acknowledged the request.

**Runs → Schedule**
- Header toggle becomes `List | Timeline | Schedule`.
- Filter pills: `All 4`, `Waiting 2`, `Due 1`, `Canceled 1`.
- Columns: `Request`, `When`, `Status`, `Model`, `Created`. Keep the current row height and type sizes.
- `When` values read `Lead decides`, `Next available` or `Nov 2, 9:00 AM PST`.
- Statuses: `Queued` (no time set), `Scheduled` (future time), `Due · waiting for lead` (time has passed) and `Canceled`.
- The selected request shows a right-side detail pane containing:
  - the full request text
  - When and Status
  - Settings, marked `Captured at send · can't change`
  - actions: `Reschedule`, `Cancel request` and `Open conversation →`
- One footer line in the pane: `Design preview: requests are saved, nothing starts.` Show it once, not on every row.
- Empty state: "No one-off requests yet." with a `New one-off` button that opens Conversation with One-off already selected.

**Key UX risks**
1. **Schedule under Runs suggests things will start on their own.** A row reading "Due · 2h ago" with nothing happening looks like the factory is broken. The status `Due · waiting for lead` and the single preview footer line make clear nothing is stuck. Also never let a request row turn into a Run status or a Sundial mark.
2. **The mode can stay switched by accident.** If One-off stays selected after sending, ordinary chat messages get queued as tasks. Reset to `Message` after every send. The meta line in the bubble makes a mistake visible, and `Cancel request` is one click away from it.
