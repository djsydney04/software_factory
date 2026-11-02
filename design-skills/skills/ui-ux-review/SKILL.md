---
name: ui-ux-review
description: "Use for evidence-based review of UI designs, mockups, prototypes or implemented products, especially compact workspaces and agent workflows."
---

# UI/UX Review

Identify what is available: static image, interactive prototype or working product. State the review boundary. Never mark keyboard access, data persistence or backend behavior as verified from an image.

## Review in task order
1. Can a new user locate their project/object and understand what needs action?
2. Can they complete the main job without confusing global, project, run and attempt scope?
3. Do navigation targets work, return context persist, and long names remain usable?
4. Do pending, empty, failed, cancelled, disconnected and stale states have useful next steps where applicable?
5. Are text, focus, status and controls legible and keyboard accessible?
6. Does the layout retain the primary job at the target widths and with realistic content?
7. Are tokens, alignment, density and hierarchy consistent without unnecessary boxes or decoration?

Run shared [checks](../ui-ux-designer/references/checks.md) against real declared inputs when available. Inspect rendered screenshots at representative widths. For implemented flows, test keyboard entry/exit, menu selection, focus return, duplicate submissions and failure recovery. Do not fabricate tests or create meaningless tests that mirror styling values.

## Report
For each actionable finding give location, evidence, user consequence, smallest useful correction and severity. Prioritize blocked work and wrong scope over aesthetic preferences. Separate verified findings from hypotheses and untested behavior. Record passed scenarios and limits briefly. Recheck affected paths after fixes; do not expand scope without a reason.

Use the suite's [behavioral cases](../ui-ux-designer/references/behavioral-tests.md) as acceptance examples. Produce a concrete revised design or patch when asked to fix, not just critique.
