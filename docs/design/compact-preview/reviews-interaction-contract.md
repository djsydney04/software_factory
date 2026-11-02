# Reviews interaction contract

This contract replaces the earlier queue/Split Reviews surface. It describes the browser-local React preview. The future factory service and GitHub integration remain separate work.

## Navigation and work modes

`#/reviews` opens the inbox. Search and status filtering narrow compact link rows. Historical merged strings are read-only history, not fabricated review records or navigation targets. `#/reviews/:reviewId` opens the selected review desk; Conversation and other review links use that stable route. An unknown ID offers recovery to the inbox instead of silently selecting another review.

Back returns to the inbox with its filter/search context and focuses the originating row when present. Previous and Next traverse all known reviews. A decision leaves the current review open. The desk has no persistent queue and no Split mode. Existing drafts and selected review state survive ordinary navigation.

Preview is the initial mode when an available screenshot belongs to the review; otherwise the desk starts in Changes. Preview uses readable-width imagery with vertical scrolling, plus native 100% image panning. It never manufactures a live or before view. Changes owns the file selector and focusable diff scroller. Evidence owns the detailed source/check/artifact/limitation record and local decision history. Tabs and controls have descriptive labels; review and diff scrollers remain keyboard reachable. Narrow layouts show one main work surface, with deliberate scrolling inside wide diffs or native-size images rather than horizontal page overflow.

## Identity and evidence

A review pins `{runId, attempt, revision}`. A Run is one execution attempt; its attempt number is a display identity, not a second mutable attempt record. The linked Run and task must exist, and its attempt/revision must still match the review. Missing or changed source is an explicit unavailable state and disables decisions.

Only checks whose revision equals the pinned review revision contribute to its pass count. Stale checks remain inspectable with their own revision. Missing, malformed, unknown, failed, pending, or running check evidence cannot certify a merge. Artifacts retain their revision and state; an artifact claiming Current at a different revision is shown as stale. Optional missing imagery and other limitations remain visible. A text-only review can remain eligible when its required current checks pass; an absent optional preview alone does not block it.

## Local actions

| Action | Conditions and outcome |
| --- | --- |
| Request changes / edit request | Requires a valid unchanged pinned source and nonblank feedback. Saves a local source-bound decision, marks Changes requested, and clears only that review's submitted draft. |
| Cancel or close feedback | Retains that review's draft for recovery. It sends no comment. |
| Simulate merge | Requires Ready state, valid unchanged source, a succeeded linked Run, no newer attempt for the same task, well-formed check evidence, and at least one current check with every current check passing. |
| Confirm a decision | Rechecks the expected source and current eligibility at commit time. A failure retains the prior state and explains recovery; stale dialogs cannot commit against changed source. |
| Repeat an already recorded action | Does not append a duplicate merge or identical change request. A merged review remains readable and cannot receive another decision. |

Decisions capture review ID, action type, Run ID, attempt, revision, timestamp, and feedback for change requests. They do not rewrite the Run or certify future revisions. Per-review feedback, status, notes, and valid decision history persist in the existing browser store. Restoration discards invalid records without filling missing historical source evidence from current defaults. Reset preview data clears the local review state with the other sample data.

The footer labels these actions as local preview behavior. No GitHub merge/comment, repository write, provider call, execution dispatch, or change to Radar follows a simulated decision.

## Verification boundary

The pure domain boundary is [review-state.js](../../../apps/desktop/src/review-state.js); [reviews.test.mjs](../../../apps/desktop/tests/reviews.test.mjs) exercises exact revision counting, malformed evidence, optional preview limits, missing source, confirmation races, successful/latest-attempt guards, restoration, independent drafts, immutable decision snapshots, and duplicate prevention. Run it alongside the existing suites using [getting started](../../getting-started.md#validate-the-frontend). Tests do not establish rendered usability. The [desktop QA record](../../../apps/desktop/design-qa.md) owns browser, focus, responsive, and screenshot evidence with its remaining limits.
