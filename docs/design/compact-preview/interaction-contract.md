# Compact run UI contract

Frontend design preview only. Keep one discreet **Design preview** label in the shell. Runs, models, permissions, delivery and checks use sample state; actions must not imply a provider ran, an OS permission changed, a file was edited or a setting reached a daemon.

## Objects and settings scope

`Project → task/conversation → runs → activity/files/checks/artifacts`.

A run is one execution attempt, with its own stable ID and immutable settings. A task can have several runs. **Retry** creates a new run linked to the same task; the UI may call it **Attempt 2**. History switches among these sibling runs. Allow one active run per task; detail always identifies the selected run. There is no separate logical-run/attempt entity.

| Surface | Scope and behavior |
| --- | --- |
| Project settings | Defaults for future submissions: provider/model, supported effort, execution mode, context inputs and file access. Saving changes does not alter existing runs. |
| Composer model control | **Next run** overrides only. Show the resolved model and effort beside the composer. Include **Use project default** and a visible inherited/override distinction. |
| Existing run → Settings | Read-only **Selected run · Effective settings**, captured when that run was submitted. Show resolved values and their source: project default or explicit override. Never rewrite these when defaults change. |
| Retry settings | Start from the failed/interrupted run's snapshot. An explicit edit applies to the new run only. Offer **Use current project defaults** explicitly; do not switch silently. |

Use a clearly labeled sample model catalog. Loading, unavailable and unsupported options need distinct states. A model change cannot silently substitute another provider or unsupported effort. Keep provider/model, effort and scope in one compact picker/popover rather than a permanent configuration panel.

## Runs and history

**Runs: List / Timeline** are alternate views of the same filtered records. Preserve the project, filters, selected run and back-navigation position when switching. List rows show title, status, model, attempt number and elapsed/update time; the title/row is a real navigation target. Timeline uses those same runs and timestamps, with a clear current-time marker. Do not draw future activity as observed work.

Run detail uses one compact header: back/breadcrumb, title and short ID, status, **History** selector for sibling runs, applicable action. Keep shared project navigation visible; do not nest a second full app shell.

| Tab | Contents |
| --- | --- |
| Activity | Short shared progress summaries and grouped, expandable tool evidence. No private-reasoning transcript. Show queued steering and failed delivery distinctly. |
| Files | Changed paths for the selected run, change counts and revision; **Open in Reviews** for the full diff. Attached context files are not changes. No merge controls here. |
| Checks | Named checks with pending/running/pass/fail/unknown result, observation time and tested revision. A previous run's green check cannot certify this run. |
| Artifacts | Preview/demo/screenshot/report links bound to the selected run and revision. Include missing, expired and stale examples. Sample artifacts remain labeled as examples. |
| Settings | Immutable effective provider/model/effort, execution mode, context revision and access scope for the selected run. Link to project settings with future-run scope stated. |

Opening an older run in History changes all five tabs and its stable run ID together. Keep its settings, activity, evidence and artifacts intact. A direct run link resolves to that same selection; back restores list/timeline and filters.

## Controls and states

Use `Queued`, `Running`, then `Succeeded`, `Failed`, `Interrupted`, `Cancelled` or `Needs approval` for runs. Verification is activity plus an evidence gate, not another board column. Tasks retain Backlog / Ready / Running / Review / Done / Blocked.

| Selected run | Available action and result |
| --- | --- |
| Queued | **Cancel** simulates cancelling this queued run. No interrupt or retry while active. |
| Running | **Interrupt** targets this exact active run; show **Interrupting…** before terminal Interrupted. Disable repeat clicks and Retry until it is terminal. |
| Failed / Interrupted / Cancelled | **Retry** reviews settings, then creates a new stable run ID for the same task, labeled Attempt N+1. Preserve all earlier evidence. |
| Needs approval | Show the unresolved request and scope. Preview may demonstrate a proposed approval UI; no actual grant. Retry stays unavailable while the request is unresolved. |
| Succeeded | View evidence / open Reviews. Use the composer for a follow-up; do not imply a successful run needs retry. |
| Unknown / reconnecting | Retain last known data with a stale indicator; disable execution controls until reconciled. Never display an unknown outcome as successful. |

Steering targets the active run only when supported. Otherwise show **Queued for next run/turn**; receipt, provider acceptance and applied evidence are different states. Preserve the message draft after a simulated delivery failure and offer a retry without duplicating accepted messages.

For the preview, transitions are local demonstrations with concise feedback such as **Preview: new run added · Attempt 2**. No fake authentication success, remote persistence or real process-cancellation claim.

## File settings are separate from file changes

Project **Files & access** separates **Context** (selected inputs, roots, exclusions) from **Edits** (Read only / Worktree edits). Attaching a file makes it an input; it does not grant write access. Excluding an input is not deleting the file. Permission changes affect future runs; they do not revoke a running process or erase historical evidence. The preview saves illustrative choices only.

## Acceptance cases

1. Change project default model: next-run inherited value updates; all existing run settings stay unchanged.
2. Override the next-run model, navigate away and return: draft and override survive; submit creates the matching effective snapshot.
3. Pick an unsupported model/effort combination: visible explanation and recovery; no silent fallback.
4. Switch List → Timeline → detail → back: same records, selection and filters; no duplicate run created.
5. Select Attempt 1 in History after retry: every tab and the stable run ID switch together; Attempt 2's result never leaks into it.
6. Retry a failed run twice rapidly: one new run with a new ID appears under the same task. Its settings match the reviewed retry configuration; the old run remains unchanged.
7. Interrupt an active run: pending state first, then Interrupted; no new run automatically appears.
8. Simulate failure/reconnect: retain stale history and draft; disable conflicting actions until state is known.
9. Change the fixture revision: prior checks/artifacts show stale and cannot present current verification.
10. Attach a context file and switch access to Read only: context remains visible, no changed file is invented and no OS permission claim appears.
11. Keyboard: logical tab order; picker opens with Enter/Space, arrows select, Escape closes and restores focus; tabs expose selected state; opening detail focuses its heading; back restores the triggering row.
12. Compact viewport: tabs and primary actions remain reachable; logs/diffs scroll within their region; collapse ancillary panes before squeezing the main content. Validate real rendering at the chosen desktop and narrow widths rather than trusting a declared layout.

Naming: **Run** is the stored record; **Attempt N** is a display label for sibling runs belonging to one task. This follows `docs/architecture.md`; no new domain entity or backend migration is implied.

Source: `ui-ux-agent-flows/SKILL.md`, its shared handoff checks, and current `docs/harness.md`, `docs/providers.md`, `docs/architecture.md` and `docs/acceptance.md`. This is an interaction handoff; runtime behavior and rendered accessibility still require testing in the preview.
