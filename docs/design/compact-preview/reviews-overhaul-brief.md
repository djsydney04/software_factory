# Reviews: inbox and full-width desk

The user asked for a total Reviews overhaul after repeatedly asking for a smaller interface, clearer hierarchy, and a larger review pane. This revision supersedes the earlier persistent review queue and Split layout. It keeps the warm-paper shell, one lead Conversation, and a separate Radar.

## Chosen direction

**Reviews** (`#/reviews`) is a compact inbox with search, a status filter, and linked rows. Selecting a review opens its own desk at `#/reviews/RV-24`. The desk uses the full workspace width; there is no permanent queue beside it. Back restores the inbox filter and row focus. Previous and Next traverse the known reviews without automatically advancing after a decision.

The desk has three work modes:

- **Preview:** one real, labeled sample screenshot at a readable 460–520px desktop width, with vertical scrolling. Native **100%** uses source pixels and permits panning inside the image scroller. Missing imagery has an explicit empty state. There is no invented before image or live preview.
- **Changes:** a full-width diff with its file picker. The picker appears only in this mode.
- **Evidence:** the pinned Run, attempt, and revision; current and stale checks; artifact states; limitations; and local decision history. A prior green check does not certify a different revision.

The footer keeps local Request changes and Simulate merge actions close to the evidence summary. Feedback drafts persist independently per review. Decisions capture their exact source and survive reload. No review action calls GitHub, sends a provider message, dispatches an agent, or modifies the Run.

## Claude consultation and selected changes

Claude Opus 5.5, through Cursor CLI in read-only plan mode, compared an inbox followed by a full-width desk with a direct desk containing a top review picker. It recommended the inbox/desk shape because it preserves an overview without permanently shrinking inspection. It also identified repeated metadata, hidden evidence, and lost feedback drafts in the prior implementation.

The implementation owner accepted that shape, separate work modes, explicit revision binding, draft recovery, and local outcome history. The owner simplified the suggested six-column inbox, chose native image pixels for 100% rather than inferred device-point scaling, and made Previous/Next traverse all known reviews. There is no automatic queue skip after a decision. The existing labeled screenshot remains a sample source; consultation did not capture new application evidence.

See [session provenance](reviews-overhaul-provenance.json) and the [interaction contract](reviews-interaction-contract.md). The [desktop QA record](../../../apps/desktop/design-qa.md) owns measured layout and executed browser checks; this brief records intent rather than test results.
