---
name: ui-ux-workbench
description: "Use for operational dashboards, project workspaces, editors and multi-pane tools where users need persistent context and frequent actions."
---

# UI/UX Workbench

Start with the recurring work, not a collection of metric cards. Define the navigation hierarchy, primary work surface and optional supporting inspector. Prefer lists, tabs and document surfaces over a card around every element.

## Composition
Use a quiet global rail, a contextual project/task list when necessary, and a flexible main workspace. Place the active object's name, status and primary action together. Let selection and content establish hierarchy. Use one clear seam between neighboring regions; avoid nested outlines and repeated headers.

Start with 200–248 px navigation, 280–360 px optional inspector, and 560 px main-content minimum as recommendations, then measure actual content. These are not observed specifications or universal breakpoints. Compactness comes from removing duplication and aligning rows, not from shrinking all text. Use 13–14 px dense metadata with 14–16 px reading text as initial candidates.

## Responsive and content rules
Budget each layout at 1440, 1280, 1024, 768 and 390 CSS px when relevant. Collapse the inspector first, then contextual navigation into a drawer, then global navigation. Preserve the primary task and object scope. On narrow screens show one work surface with explicit back navigation; retain selection and draft state across pane changes.

For tables, assign minimum widths and priorities before drawing columns. Keep object identity, state and next action visible. Move secondary metadata to detail; allow deliberate horizontal scrolling for truly tabular comparison. Never hide critical errors only in truncated text. Provide an accessible way to reveal full names; hover alone is insufficient.

## Deliver
Provide pane sizes and collapse order, a row/column budget, navigation/selection behavior, primary and secondary actions, empty/loading/error states and long-content examples. Use the shared layout checker described in [checks](../ui-ux-designer/references/checks.md). Verify the actual rendered layout afterward. Avoid cinematic heroes, decorative statistics and large empty greeting panels inside an operating workspace.
