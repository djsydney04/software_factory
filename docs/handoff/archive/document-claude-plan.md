I'll read the desktop agent guide and the current Scratchpad sources, then give a short critique.The direction is right: one native, autosaved document with highlights behind it is the smallest change that fits the request. The main gap is that "done" has no data to come from yet.

**1. Surface anatomy**
- **Header (40px):** "Scratchpad" on the left. On the right, a 13px muted save state ("Saved locally", or "Not saved, storage unavailable") and a quiet count such as "2 done". That count is a button that moves the cursor to the next done span. Nothing else goes in the header.
- **Body:** one textarea filling the pane, with no border, box or footer. Keep 14px/22px text, a max line length of about 72 characters, and 24px padding. The placeholder should read "Start writing…".
- **Settings > Scratchpad:** cadence, time, zone, pause and the cron line, in the existing field pattern.

**2. Risks and corrections**
- **Nothing can ever highlight yet.** `scheduling.js` has no attempts, only request `status` values: pending, scheduled and cancelled. Nothing can truthfully be "succeeded" in this preview. Show zero highlights rather than seeding fake successes. If you need example data to show the look, label it as an example.
- **Highlights will drift or vanish.** Store the span's exact quote plus a little surrounding text (prefix and suffix), not just character offsets. Re-find it only when there is exactly one match; otherwise drop the highlight. Plain offsets fail closed after any edit earlier in the document, so highlights would disappear far too often. The mirror behind the textarea must copy its font, padding, wrapping (`white-space: pre-wrap`), scroll position, scrollbar width and resize behaviour. It must also skip re-rendering while an input method (IME) is composing.
- **Triage is not a setting.** Moving Preview triage into Settings separates the action from the text it acts on. Keep it in Settings only if the user accepts that. When migrating, flatten the old notes in creation order, keep unsent drafts and edits as text, and keep each note's original identity, revision and request link as internal provenance. Being sent does not mean done.

**3. Keyboard, accessibility and truthful-state checks**
- The mirror is `aria-hidden`, so screen readers need another route. The "N done" button should select the next span and announce it, e.g. "Done: …, request REQ-…".
- Don't mark done with color alone. Use the sage tint plus an underline, and check forced-colors mode, where background tints disappear.
- Tab must still leave the textarea. Don't capture ⌘/Ctrl+Enter, and leave undo and redo native.
- Announce save state politely and debounced, never on every keystroke.
- Cancelled, failed, queued and submitted requests never highlight. Deleting text removes its highlight for good.
- The Settings copy must still say that no cron or agent runs.