# Scratchpad and permissions contract

Contract for the current local frontend. The [QA record](../../../apps/desktop/design-qa.md) records executed checks. The browser preview has no agent runtime, active cron, or permission enforcement.

## Writing surface and controls

Scratchpad is one blank, continuous document. It autosaves locally while typing; there is no Add, Send, note list, or lifecycle filter. A minimal title, save state, and completion evidence surround the writing area. Native multiline editing, selection, undo, and Tab navigation remain available. A storage failure must replace the saved claim with a recovery message.

**Settings → Scratchpad** (`#/settings/scratchpad`) owns every review control and manual **Preview triage**. Default to **every morning at 09:00** in the browser's captured IANA zone. Allow daily, hourly, or every 15 minutes, applicable time, zone, and pause intent. Show the saved human schedule and derived cron. Invalid configuration leaves the last saved plan intact. **Runtime disconnected** means the plan does not run; pause records the user's intended behavior for a future connected runtime.

Conversation remains separate. Its native **Type: Message / One-off** select stays at the bottom beside Send. One-off timing and help share that bottom action section, below model and permissions; on narrow screens timing wraps above Type / Send. Preserve the draft when navigating or changing type.

## Preview passages and source identity

Blank lines separate preview passages. **Preview triage** in Settings saves each new, nonblank, unlinked passage as a **Lead decides** Request. It copies source text, current project defaults, and current project file context. An unsent composer override cannot affect it. Existing unchanged source ranges are excluded, so repeated triage or cancellation does not duplicate their origin. The document stays unchanged.

Each source anchor identifies a retained note ID/revision and its exact document range. Submitted Requests preserve their original text and settings/context; document editing never rewrites them. Edits before an untouched range shift its position. An edit that overlaps source text invalidates its anchor. Malformed, overlapping, or ambiguous restored anchors are discarded. Do not recover an edited anchor by matching the same words elsewhere.

A source link in Runs → Schedule returns to the document and selects its retained range. If the source has changed, show that the Request keeps its original text. Editing a previously submitted passage can produce new intent on a later explicit preview; it does not update or replace the historical Request.

## Completion evidence

A green passage requires all of the following: an exact valid source anchor, its matching note/revision and Request provenance, unchanged Request text, a non-cancelled Request, and a latest linked Run whose status is **succeeded**. Use the Run's `requestId` association; unrelated sample successes cannot certify a passage. A later queued, running, failed, interrupted, or cancelled attempt removes the earlier success highlight. Submitted or queued intent alone never means completed work.

The highlight mirror is hidden from accessibility APIs; the native textarea remains the editable surface. Completion details identify the completed text and link to its supporting Run. Preserve readable selection and a non-color completion cue. With no linked successful execution, render no completed highlight. This preview must not seed successes or claim an agent actually completed work to demonstrate the treatment.

## Migration and storage

Convert existing visible notes in creation order into document text once. Use unfinished edit text where present and append the unfinished capture draft without trimming it. Retain original note IDs, revisions, request links, edits, draft, and archived records as internal provenance. Archived text remains historical rather than appearing as new document work. Once the document exists, reload uses its canonical text; clearing it must not reappend legacy material.

Persist the document, source anchors, configuration, provenance, and Requests in the local aggregate. Promotion is atomic: invalid choices or identity collisions leave it unchanged. Storage failure retains in-memory text but cannot promise reload persistence.

## Permissions and runtime boundaries

Keep **Access** as `read-only` or `worktree`. Independent **commands**, **network**, and **integrations** permissions use `ask`, `allow`, or `deny`, defaulting to `ask`. Project defaults set ceilings; next-send overrides can only narrow them. Workflow Mode cannot override a denial. New Messages, Requests, and current-default retries capture resolved settings; original-basis retries preserve the original snapshot. Legacy evidence remains **Not captured**.

**Settings → Permissions** (`#/settings/permissions`) names the project, explains each capability, and saves with **Save project permissions**. The composer's Permissions dialog shows those saved rules separately from editable next-send restrictions and links directly to the project editor. Direct entry focuses the selected Permissions tab; Cancel and Escape return focus to the composer trigger. Saving defaults does not rewrite prior snapshots.

Routine assignment updates may collapse in Conversation; approval requests and review outcomes stay visible. Each own send has one local receipt rather than a duplicate success notice. New messages follow the bottom only when the user is already there or has just sent; otherwise preserve reading position and offer **Latest updates**. Smooth scrolling respects the reduced-motion preference. None of these presentation changes simulates an agent response.

Triage fabricates no execution Run, worker task, Radar mark, agent interpretation, or lead acknowledgment. A future connection still needs durable scheduling, validated capability enforcement, agent triage, lead ordering, dispatch, missed-slot recovery, and provenance reconciliation. Cancelling intent must prevent future dispatch without implicitly cancelling an existing attempt. Preview settings grant no OS or provider access.

See the [one-off contract](oneoff-interaction-contract.md), [run contract](interaction-contract.md), and [document consultation](document-claude-brief.md).
