# Prototype handoff checkpoint

The shared routing guide arrived after implementation and independent review. Existing work is preserved. The remaining scope is local verification and handoff. No PR, deployment, or engine implementation is part of this design pass.

1. Scope the decision the prototype exists to make: which layout, which interaction, which density, or for an empirical fork which behavior, timing, or approach. No decision means no prototype. Route to Feature.
   Complete. The user is choosing compact factory density and run/settings interactions.
2. Gather references when the design space is open. Search for prior art, summarize a moodboard of themes, palettes, and layouts, let the user pick directions before building. Skip when the direction is set.
   Complete before this checkpoint. The existing Claude screens and later user feedback define the direction.
3. Build throwaway in an isolated scratch dir, separate from production source. For a visual decision, vanilla HTML/CSS/JS or the lightest stack that renders the idea, CDN deps, a dev server with hot reload. For a behavioral or timing decision, the smallest script that exercises the question. No production framework, no tests, no abstractions.
   Complete using the already-authorized Product Design React template in apps/desktop. It is a frontend-only design artifact, separate from the unimplemented Rust engine. The late guide does not justify replacing finished work.
4. When comparing alternatives, build them behind one switcher (buttons or a keypress), each variant labeled. This is the **exhaust-the-design-space** principle skill made cheap.
   skip: The user selected refinement of the existing direction, not another alternatives round.
5. Verify on the matching surface. For a visual decision, screenshot each variant via the control skill and drive the interaction. For a behavioral or timing decision, observe the thing you are deciding by logging the timing, printing the output, or watching the render. The observation is the test here, not an assertion.
   Complete. Real browser controls, 35 layout cases, state-paired captures, and source comparisons are recorded in design-qa.md.
6. Present alternatives, tradeoffs, and a recommendation. The output is the decision plus the throwaway artifact, not shippable code. Hand the chosen direction to **Feature** (or `architect` for the shape) for the real build.
   Complete at handoff. Recommend the compact hierarchy as the next product direction. Production implementation remains separate.
