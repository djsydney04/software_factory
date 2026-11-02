# Conversation polish pass

I looked at `conversation-before.png`. The main problems in it are the long outlined Next-send pill, an avatar on every Factory message, a weak Message/One-off selected state, and approval and review rows that look like ordinary text. All edits go in [apps/desktop/src/App.jsx](../../../apps/desktop/src/App.jsx) and [apps/desktop/src/styles.css](../../../apps/desktop/src/styles.css). Behavior, localStorage, native selects and the 14px body / 13px metadata sizes stay the same.

## 1. Calmer composer
- `.composer`: change `border-color` from `--boundary` to `--separator`. Keep the existing `:has(textarea:focus-visible)` outline.
- `.picker-btn`: make `border-color` transparent. On `:hover` and `[aria-expanded="true"]`, use `background: var(--inset)` and `border-color: var(--separator)`. Keep the "Next send" `.scope` label so the scope stays explicit.
- `.picker-btn .src.ovr`: add `padding: 0 4px; border-radius: 4px; background: var(--inset)`. "Default" remains plain muted text.
- `.composer-intent`: add `border-top: 1px solid var(--separator); padding-top: 6px`. This separates intent from the draft text.

## 2. Message hierarchy without repeated avatars
- In `Conversation`'s `messages.map((m, i) => ...)`, add a `cont` class to `msg lead` when `messages[i - 1]?.role === "lead"`.
- On continuation messages, keep `.avatar` in the layout but set it to `visibility: hidden`. Give `.who` the `sr-only` class there so screen readers still hear "Factory".
- `.msg.lead.cont { margin-top: -8px; }`. Grouped lead messages end up 8px apart, and turns stay 16px apart.
- `.messages { min-height: 100%; justify-content: flex-end; }`. The thread sits on the composer instead of floating at the top, and no filler content is added.

## 3. Clearer selected states
- `.intent-toggle button[aria-pressed="true"]`: set `box-shadow: 0 0 0 1px var(--boundary); font-weight: 500`.
- `.nav-row[aria-current="page"]`, `.queue-item[aria-current="true"]`, `.tree-file[aria-current="true"]` and `.vtabs [aria-selected="true"]`: add `box-shadow: inset 2px 0 0 var(--action)` on top of `--selected`. This marks selection by shape as well as tint.

## 4. Review and approval handoffs
- On the lead `li`, add `ask` when `m.approval` is set and `handoff` when `m.review` is set.
- `.msg.lead.ask .btn-row`: `padding: 6px 10px; border-left: 2px solid var(--amber); background: var(--raised); border-radius: 0 6px 6px 0`. Scope this rule under `.msg` so the shared `ApprovalActions` styling in Runs and Radar stays unchanged. Allow and Skip keep equal weight so neither choice is nudged.
- `.review-link`: `padding: 0 10px; border: 1px solid var(--separator); border-left: 2px solid var(--sage); border-radius: 6px; background: var(--raised)`. Sage matches the ready `.q-dot` in Reviews.

## Verify
Run the dev server and take a new conversation screenshot to compare with the before image. Check Message and One-off, an override, the approval decided state, a 768px width, and that Runs and Radar approvals look the same as before.

## Parent decisions

Keep the composer boundary token to retain a visible text-input boundary. Move intent/timing above the draft, shorten the collapsed model summary while retaining next-send scope and exceptional modes, and keep the full settings in its accessible name and expanded panel. Apply message grouping, bottom anchoring, stronger selected markers, and restrained handoff emphasis. Do not remove request status from its inspector: source shows only one detail status row. No data-model or backend changes.
