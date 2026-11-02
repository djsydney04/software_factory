# Blank-document Scratchpad refinement

The user requested one blank document, highlights for work completed by the agent, and review settings moved to Settings. This direction supersedes the earlier note-list Scratchpad layout. It preserves the compact shell, one Factory lead conversation, and separate execution and request identities.

## Bounded Claude consultation

Claude reviewed the direction through authenticated Cursor CLI in read-only plan mode with the sandbox enabled. Requested model: `claude-opus-5-5-high`; effective model: **Claude Opus 5.5 300K High**. The session completed successfully in 28.8 seconds, exit 0. See [provenance](document-provenance.json).

The public critique supported one native autosaved writing surface with minimal save status. It highlighted mirror alignment, wrapping, scrollbar and scroll-position parity, native undo/IME behavior, accessible completion evidence, and a non-color completion cue. It emphasized that submitted or queued text must never count as done. These are design recommendations; the [QA record](../../../apps/desktop/design-qa.md) owns executed verification.

## Local implementation decisions

- Use one continuous document, without capture buttons, note cards, or lifecycle filters. Keep schedule configuration and the manual preview action together in **Settings → Scratchpad**.
- Preserve exact source anchors through surrounding edits; invalidate overlapping edits and malformed or ambiguous restored anchors. Claude suggested quote/context recovery, but the local implementation deliberately avoids reattaching changed source to repeated words elsewhere.
- Require a matching Request and latest linked succeeded Run for green passages. Claude's warning that Requests have no success state was valid; completion evidence comes from linked Runs, not request status. No successful work is fabricated.
- Retain legacy note/revision/request provenance and migrate visible text, unfinished edits, and drafts once. Editing the document leaves submitted Requests intact.
- Keep the daily 09:00 local-zone plan configurable. Saving it does not install cron or connect an agent runtime.

The [interaction contract](scratchpad-permissions-contract.md) describes current behavior. This consultation was not an implementation run, live agent dispatch, or proof of completed product work.
