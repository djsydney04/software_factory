---
name: ui-ux-agent-flows
description: "Use when designing agent products with conversation, delegated work, asynchronous runs, model choices, approvals or artifact review."
---

# UI/UX Agent Flows

Model the objects before drawing: workspace → project → conversation/task → run → attempt → artifact/check. Adapt names to the product, but keep identity and scope unambiguous. Project conversation, aggregate progress and review are complementary views of the same work.

## Conversation and controls
Keep the composer close to context attachments and model/mode choices. Distinguish inherited project defaults, explicit next-run overrides and the immutable effective settings of an existing run. Show the resolved value and scope; changing a picker must not silently rewrite history. Use actual available options or mark illustrative choices clearly. Include unavailable/loading and unsupported-option states.

Summarize activity with expandable evidence rather than dumping every internal event. Separate user-visible progress from hidden reasoning. Preserve drafts, scroll position and selected project. Expose pending messages and delivery failures with recovery.

## Runs and review
Make run names/rows real navigation targets with stable identifiers. Run detail includes status, start/update times, goal, effective settings, attempts, activity, changes, checks and artifacts. Breadcrumb/back restores project and filter context; direct links resolve to the same run. Label stale data and reconnection state.

Specify the state machine, for example queued → running → succeeded/failed/cancelled, with waiting-for-input and interrupted behavior where supported. Describe enabled actions in each state. Distinguish steering the active attempt, interrupting it, retrying as a new attempt and creating a new run. Prevent duplicate submissions and conflicting active attempts. Show confirmation only when consequences justify it; never invent backend support.

Review links evidence to the exact run/attempt and revision. A past green check must not certify a later revision. Give accept/request-changes actions clear scope, pending feedback, error recovery and auditable results. If only a mockup, label the actions as proposed.

## Deliver
Provide object/scope map, happy path plus failure/reconnect/cancel paths, control availability table and interaction contract. Specify keyboard access, focus movement and return navigation. See the shared [handoff check](../ui-ux-designer/references/checks.md); a complete declaration still needs runtime tests.
