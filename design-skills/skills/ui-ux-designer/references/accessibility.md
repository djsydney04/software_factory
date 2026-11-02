# Accessibility checks

Use current primary guidance when making compliance claims. These checks are a baseline, not a certification.

- [Text contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html): 4.5:1 ordinary text; 3:1 large text (24 CSS px regular or approximately 18.67 CSS px bold). Compare unrounded ratios.
- [Non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html): applicable control boundaries, states and meaningful graphics need 3:1 against adjacent colors. Decorative seams are different from boundaries needed to identify a control.
- [Target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html): 24×24 CSS px minimum subject to the standard's spacing and other exceptions. Prefer larger targets for touch and frequent actions. Row height alone does not establish target size.
- [Combobox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/): use native select where suitable; otherwise specify label, expanded state, focus, arrow navigation, selection and Escape behavior for the chosen pattern.

Check visible keyboard focus, logical reading/focus order, labels, error association, zoom/reflow, reduced motion and color-independent status. Do not transfer hover-only affordances to touch. Static mockups cannot prove these behaviors. Alpha colors, gradients and text over media need actual composited/rendered sampling; the bundled checker accepts opaque sRGB only.
