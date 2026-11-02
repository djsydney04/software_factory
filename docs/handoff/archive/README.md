# Public authoring history

These are selected public final briefs, plans, review records, checkpoints, and image prompts from the original task's `work/` directory. They preserve how this design evolved. They are historical evidence, not a fresh assignment, runnable setup instructions, proof of live execution, or permission to invoke a provider.

## Precedence

Follow the current [prototype instructions](../../../apps/desktop/AGENTS.md), [design overview](../../design/README.md), [current design handoff](../../design/compact-preview/README.md), interaction contracts linked there, and the latest sections of the [QA record](../../../apps/desktop/design-qa.md). Those sources and the current implementation supersede this archive. The [original user objective and references](../source/README.md) retain the broader intended product scope.

In particular, earlier embedded Reviews panes and persistent queues/Split views, note-card Scratchpads, proposed 08:00 defaults, earlier composer positions, and any initial Message-to-Run behavior are obsolete. Current behavior uses one lead Conversation, separate Inbox/Review desk, a blank document with Settings-owned configurable 09:00 review plan, bottom composer type/timing controls, project permission ceilings, and separate Request intent versus execution Runs. No source in this archive establishes a connected scheduler, enforced permission policy, working Rust daemon/TUI/native app, or live GitHub integration.

## Included files and original provenance

Original locations below are relative to the original task root, whose `outputs/software-factory` now points to the canonical project checkout. Keeping task-relative provenance avoids an unnecessary dependency on the original machine's absolute path. Text is preserved except that repository-relative Markdown links in two plans are normalized to resolve from this directory; plain historical paths and command examples remain historical text.

| Included file | Original source | Role |
| --- | --- | --- |
| [cursor-claude-redesign-brief.md](cursor-claude-redesign-brief.md) | `work/cursor-claude-redesign-brief.md` | Original public Claude redesign brief; later compact dimensions and Reviews flow supersede it. |
| [cursor-claude-redesign-prompt.md](cursor-claude-redesign-prompt.md) | `work/cursor-claude-redesign-prompt.md` | Bounded read-only redesign assignment and original reference selection. |
| [cursor-claude-radar-refinement-prompt.md](cursor-claude-radar-refinement-prompt.md) | `work/cursor-claude-radar-refinement-prompt.md` | Correction excluding Reviews content from the Sundial Radar. |
| [cursor-claude-compact-build-prompt.md](cursor-claude-compact-build-prompt.md) | `work/cursor-claude-compact-build-prompt.md` | Original Claude frontend authoring assignment; historical, not a production runtime setup script. |
| [cursor-oneoff-prompt.md](cursor-oneoff-prompt.md) | `work/cursor-oneoff-prompt.md` | Read-only consultation brief for one-off intent and lead scheduling. |
| [cursor-polish-prompt.md](cursor-polish-prompt.md) | `work/cursor-polish-prompt.md` | Read-only polish consultation brief for density and hierarchy. |
| [cursor-scratchpad-prompt.md](cursor-scratchpad-prompt.md) | `work/cursor-scratchpad-prompt.md` | Original note-based Scratchpad/permissions consultation brief, superseded by blank document. |
| [document-claude-prompt.md](document-claude-prompt.md) | `work/document-claude-prompt.md` | Read-only consultation brief for the blank-document Scratchpad. |
| [reviews-claude-prompt.md](reviews-claude-prompt.md) | `work/reviews-claude-prompt.md` | Read-only consultation brief for the full Reviews overhaul. |
| [document-claude-plan.md](document-claude-plan.md) | `work/document-claude-plan.md` | Public blank-document critique and suggestions; current contract records owner decisions. |
| [scratchpad-claude-plan.md](scratchpad-claude-plan.md) | `work/scratchpad-claude-plan.md` | Public early note-based plan; source shape, grouping, and placement suggestions are historical. |
| [reviews-claude-plan.md](reviews-claude-plan.md) | `work/reviews-claude-plan.md` | Public inbox/desk proposal; current contract records simplified columns, navigation, and native-pixel zoom. |
| [oneoff-contract.md](oneoff-contract.md) | `work/oneoff-contract.md` | Original one-off interaction proposal; current published contract and composer placement win. |
| [oneoff-review.md](oneoff-review.md) | `work/oneoff-review.md` | Independent one-off source review and resolution record. |
| [scratchpad-contract.md](scratchpad-contract.md) | `work/scratchpad-contract.md` | Original note-based proposal, including rejected shapes/defaults; current blank-document contract wins. |
| [scratchpad-review.md](scratchpad-review.md) | `work/scratchpad-review.md` | Independent early Scratchpad/permissions source review and findings at that point. |
| [oneoff-checkpoint.md](oneoff-checkpoint.md) | `work/oneoff-checkpoint.md` | One-off implementation scope, state boundaries, and verification plan. |
| [scratchpad-checkpoint.md](scratchpad-checkpoint.md) | `work/scratchpad-checkpoint.md` | Early note-based implementation checkpoint and completed QA scope. |
| [document-checkpoint.md](document-checkpoint.md) | `work/document-checkpoint.md` | Blank-document implementation checkpoint and completed QA scope. |
| [reviews-checkpoint.md](reviews-checkpoint.md) | `work/reviews-checkpoint.md` | Reviews overhaul implementation checkpoint and completed QA scope. |
| [prototype-handoff-checklist.md](prototype-handoff-checklist.md) | `work/prototype-handoff-checklist.md` | Original compact-preview scope and handoff checkpoint. |
| [claude-design-image-prompt-1.txt](claude-design-image-prompt-1.txt) | `work/claude-design-image-prompt-1.txt` | Initial image-render prompt retained as authoring history. |
| [claude-design-image-prompt-2.txt](claude-design-image-prompt-2.txt) | `work/claude-design-image-prompt-2.txt` | Image-render prompt retained as authoring history. |
| [claude-design-image-prompt-3.txt](claude-design-image-prompt-3.txt) | `work/claude-design-image-prompt-3.txt` | Image-render prompt retained as authoring history. |
| [claude-design-image-prompt-4.txt](claude-design-image-prompt-4.txt) | `work/claude-design-image-prompt-4.txt` | Image-render prompt retained as authoring history. |
| [claude-radar-final-prompt.txt](claude-radar-final-prompt.txt) | `work/claude-radar-final-prompt.txt` | Refined Sundial image-render prompt retained as authoring history. |

## Already retained elsewhere

Generated mock images, screenshots, accessibility snapshots, measured geometry, tokens, source attribution, and curated provider/session provenance are already in `docs/design/` and `apps/desktop/design-qa.md`; they are not duplicated here. The original `work/run-ui-contract.md` is byte-identical to the published [Run interaction contract](../../design/compact-preview/interaction-contract.md). The original Reviews before screenshot and AX text are already in [Reviews evidence](../../design/compact-preview/reviews-screenshots/before-1440.png). The useful paused-goal probe result is already published under [experiments](../../experiments/codex-paused-goal.json) and explained in [provider notes](../../providers.md).

## Excluded material

Raw Cursor/Claude `.jsonl` streams, `.stderr` logs, and per-process status files are excluded; use the curated public provenance already published with the design. Provider authentication, account caches, local browser state, the isolated Codex home (SQLite databases, journals, sessions, installation identifiers), temporary source stages/patches and proof-fixture code, generated build/dependency caches, and private reasoning are excluded. The generated Codex app-server schema snapshot is omitted because it is reproducible version-specific metadata; the provider notes describe the observed contract and its limits. Runner/extractor scripts with machine-specific paths and old declaration-check scaffolding are not needed to build or use the current handoff.
