# Composer footer, Permissions, and Scratchpad

## Recommendation: Shape A
Shape B keeps a segmented toggle, which is the treatment the user rejected. It also hides Scratchpad inside a Conversation tab and adds a second control row. Shape A is quieter: one footer row, native selects, and Scratchpad as a separate page that never interrupts the one lead conversation.

## Composer layout ([apps/desktop/src/App.jsx](../../../apps/desktop/src/App.jsx) lines 512–630)
- **Order, top to bottom:** textarea, then attachment chips, then any expanded panel (Next send or Permissions), then the One-off timing row (only when Type is One-off), then the footer.
- **Footer (32px controls, 8px gap):** `[Paperclip] [Model · Effort ▾] [Permissions: Ask ▾]`, then flexible space, then `[⌘/Ctrl+Enter] [Type: Message ▾] [Send]`.
- **Type selector:** a native `<select>` labeled "Type" with Message and One-off. It replaces `.intent-toggle`, and that CSS gets deleted.
- **Timing row:** keep the existing When select, date-time input and `#oneoff-timing-help`, moved under the textarea.
- **768px and narrower:** the footer wraps into two rows. Row 1 holds Paperclip, Model and Permissions, which shrink with ellipsis. Row 2 holds Type and Send, right-aligned. The keyboard hint stays hidden.
- **560px and narrower:** the Permissions button shows only "Ask", "Custom" or "Allow". Panels collapse to one column.
- **Text size:** raise `.next-run-grid label > span` from 12px to 13px.

## Permissions
- A Permissions panel styled like the existing Next send fieldset, with four rows: Edit scope (worktree), Commands, Network, Integrations. Each row is an Ask / Allow / Deny select. Default is Ask for all four.
- Execution Mode stays only in the Next send panel.
- Footnote: "Requested from the lead. This preview changes no OS or provider permissions."
- Captured into `settings.permissions` when you submit. `createRequest` already uses `structuredClone`, so snapshots stay immutable. `restoreRequests` in [apps/desktop/src/scheduling.js](../../../apps/desktop/src/scheduling.js) also validates the four values.

## Scratchpad
- New sidebar entry directly below Conversation in `NAV`, with no count badge.
- **Header line (13px):** "Morning review · Every day at 09:00 · {localZone()} · Edit". Edit opens a single row inline:
  - a Cadence select: Every morning (default) or Weekdays
  - a time input (default `09:00`)
  - the browser zone shown read-only
  - the equivalent cron expression as metadata (`0 9 * * *` or `0 9 * * 1-5`)
- **Status (always visible):** "Not active in this preview. No agent sorts notes."
- **Capture:** a one-line input with an Add button (Enter also adds), followed by a newest-first list. Each note shows its text, the time added and a remove button.
- **Preview triage:** a button that groups notes using a local, deterministic rule. The result is labeled "Example triage, not agent output". Each group becomes a proposed request with the policy "Lead decides".
- **Send to lead:** one button, "Send N proposals to lead", with no per-note approvals. It calls `createRequest` with permissions and context snapshots, marks the notes with their request IDs, and posts a single conversation note. It creates no Run and no acknowledgment from the lead.
- Stored in the app's localStorage as `scratchpad: { notes, review: { cadence, time, timeZone } }`, with validation on restore.

## Unchanged
- Reviews, the active-only Sundial, the 168px sidebar, the 40px header, and the 14px body / 13px metadata text sizes.

## Docs
- Record in [apps/desktop/AGENTS.md](../../../apps/desktop/AGENTS.md) that Type moves to the footer, Permissions are separate from execution Mode, and Scratchpad defaults to every morning at 09:00 in the browser-local zone.