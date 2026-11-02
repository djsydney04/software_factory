You are the lead product designer redesigning Software Factory. The user specifically asked for Claude through the Cursor CLI, so your design judgment is the requested work. This is a DESIGN task, not product implementation. Work read-only. Do not scaffold or edit app code, install anything, commit, publish, call external integrations, or change configuration. Do not ask for clarification: the brief below is complete.

Read README.md and docs/design/README.md for context, but treat the latest user correction below as authoritative over old design requirements. Inspect the actual PNGs using your image-reading capability:
- docs/design/compact-conversation.png
- docs/design/compact-graph.png
- docs/design/compact-radar.png
- /Users/dylanmitic/.codex/attachments/decce846-2b0e-47a4-8a22-4badd380accf/image-1.png (factory.ai inspiration, do not copy)
- /Users/dylanmitic/.codex/attachments/decce846-2b0e-47a4-8a22-4badd380accf/image-3.png (Codex reference)
If image access is unavailable, explicitly state that limitation instead of pretending you inspected them.

LATEST USER FEEDBACK (verbatim): "I think we are getting better, but I want things more compact, I want there to be a clear hierarchy, I don't want changes to be in radar etc and I want radar to have our own sort of touch of design to it. Have claude redesign this stuff through the cursor CLI."

Previously clarified: 'the top' meant TOP-LEVEL OVERVIEW. The human talks to ONE main agent, not individual worker inboxes. They need a Progress area (Graph/Board/Radar) showing who works on what, and a large review experience with actual app preview/diff/checks/merge. But reviews/changes must now have their OWN destination, not be embedded in Radar or other monitoring pages. Existing warm off-white/pale stone/charcoal colors are liked. Avoid adding features, noisy metrics, AI dashboard clutter, pills everywhere, large headings, unnecessary help copy or decorative cards. Keep macro goal and current focus available. Compact means less chrome, better hierarchy and purposeful density, not illegibly tiny text. Body text 14px minimum; secondary metadata can be 12-13px. Desktop viewport 1440x900; eventual TUI/Rust desktop app should share the conceptual IA.

Deliver ONE coherent redesigned product direction spanning THREE screens: (1) Conversation; (2) Progress with Radar selected (Graph/Board still discoverable); (3) Reviews, with a large selected-change preview. Give Radar a distinctive original design gesture that communicates actual factory state/agent activity rather than copying Factory.ai, making arbitrary scores, or using a generic KPI dashboard. Keep it implementable with code-native data visualization (such as Canvas/D3) later. No code required now.

Return in Markdown, in this exact structure:
1. 'Design diagnosis' (5 short concrete bullets about current images).
2. 'Hierarchy and rules' (navigation + parent/child structure, 8 rules maximum).
3. 'Design tokens and geometry' (specific typography, sidebar/topbar sizes, spacing, surfaces, colors, active/focus treatment).
4. 'Screen specifications' (for each screen: composition, visible content, primary action, progressive disclosure; exact dimensions/proportions. Make clear what is excluded).
5. 'Radar concept' (name, distinctive visual signature, mapping of actual state to geometry, quiet motion/reduced-motion behavior; no fake progress percentage/quality score).
6. 'Image prompts' with exactly THREE separately headed fenced text prompts. Each should be detailed enough for an image model to render a single realistic 1440x900 app screenshot. All three share exact app shell and style. Label them by screen, not option number. Attachments are style references, not assets to imitate literally. The Conversation and Radar must not contain a Changes list, PR preview pane, diff or merge controls. Reviews should use most workspace for the selected app preview/diff. Use realistic minimal sample data for Green Room, avoid fake claims of real runtime evidence. Current user-facing date is November 1, 2026; avoid unnecessary dates.
7. 'Implementation handoff' (short, no app code, note any uncertainty).

Do real design work. Do not merely repeat these requirements. We will use YOUR screen specifications and image prompts to produce the next visual review set, then inspect against the user request. Keep the brief concise and directly useful.
