# Behavioral acceptance cases

Run these as scenario-based review prompts for a design agent. The expected behavior is a rubric, not a claim that live product interactions have been tested.

1. **Failed-run drilldown:** “Open a failed run from the project overview.” Use workbench and agent flows. Preserve project/run/attempt identity, define back behavior, evidence revision and retry preconditions. In the soft-charcoal profile, keep text navigation and no emoji. Fail if an unscoped modal or historical model mutation appears.
2. **Light-theme counterexample:** “Design settings and billing for a light clinic scheduling app used at the front desk.” Respect light colors, explicit labels and appropriate touch targets. No charcoal profile, agent attempts or three-pane layout unless the actual job needs them. Fail if source aesthetics override the brief.
3. **Editorial launch:** “Design a launch page for a command-line developer tool.” Route editorial and visual system; show proposition, real product evidence and an appropriate install example. Use original copy and verified claims. Fail if the hero is a generic KPI dashboard or proof is invented.
4. **Flawed capture:** “Extract patterns from screenshots with a consent banner and half-faded content.” Mark artifacts, lower confidence and recapture if available. Fail if low contrast becomes a style rule or mobile/keyboard behavior is claimed from desktop screenshots.

For implementation review, add real navigation, keyboard, reconnect, failure recovery and duplicate-action tests appropriate to the product. Record which are executed versus proposed.
