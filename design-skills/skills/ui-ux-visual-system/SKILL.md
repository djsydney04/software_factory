---
name: ui-ux-visual-system
description: "Use for creating or refining a coherent product visual system, including color, typography, spacing, density, focus and motion."
---

# UI/UX Visual System

Build semantic tokens before styling individual screens: canvas, surface, raised surface, primary/secondary text, decorative separator, interactive border, accent, focus and status. Choose light or dark from context and user direction. Use system sans for prose and controls, monospace only for identifiers, counts or technical metadata. Do not use a source's proprietary font as a shortcut to resemblance.

## Hierarchy and softness
Favor regular/medium weights and a small type scale. Start with 4/8 spacing rhythm, compact rows, 6–10 px control corners and 10–14 px panel corners, then adapt to the product. Soften through tonal adjacency, restrained borders, breathing room at groups and readable text. Avoid turning every surface into a floating rounded card. Keep true interactive boundaries and focus visible.

Use a restrained accent for selection and primary action; keep warning/error/success semantic. Pair status color with text or an icon. Emoji are governed by the project profile, never required decoration. Use icons consistently with accessible names.

## States and accessibility
Define default, hover, focus-visible, pressed, selected, disabled and loading states for controls. Distinguish selection from keyboard focus. Prefer native controls; custom selectors require complete keyboard behavior. Reserve animation for explaining transitions, with reduced-motion alternatives and no layout jumps. Start with short 120–180 ms transitions as a recommendation, not a requirement.

Check actual foreground/background pairs: ordinary text 4.5:1, large text 3:1; applicable UI boundaries and state indicators 3:1. Decorative separators may be subtle when they convey no required information. Do not use muted text as a euphemism for unreadable text. Read [accessibility](../ui-ux-designer/references/accessibility.md) and run the shared contrast checker.

## Deliver
Provide tokens with usage, type and spacing scale, component state examples and a rendered dense-content example. Supply an alternate theme only when needed to validate portability. The optional [soft-charcoal profile](../ui-ux-designer/references/soft-charcoal.md) is a starting point, not the suite's universal theme.
