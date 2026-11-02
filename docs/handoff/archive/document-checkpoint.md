# Document Scratchpad checkpoint

User direction: one blank continuous document, highlight work completed by the agent, move review settings into Settings. This supersedes the earlier note-list interface.

Implemented in the local React preview: canonical autosaved document with exact source anchors, lossless one-time legacy migration, native beforeinput mapping, document-only native Undo/Redo snapshots, green completion ranges only for the latest exact linked succeeded Run, accessible completion evidence, Settings→Scratchpad configuration/manual triage. No live runtime, cron, or provider dispatch.

Domain owner: document_domain. Root owns UI/integration/styles. Independent document_contract_review found repeated-text and Undo identity P2 issues; both fixed and final review has no remaining P1/P2. Claude through Cursor CLI supplied read-only plan consultation, effective Opus5.5 High, success; public brief/provenance copied into docs/design/compact-preview/document-*.

Evidence: all32 tests pass, production build passes,15 responsive geometry cases pass at five widths, native duplicate deletion/Undo/Redo retains exact consumed identity, source-link selection and migration/blank reload pass. App console clean. Temporary example-completion harness had HMR-only root warnings and was removed; fixtures never wrote storage. Original user data preserved, tab left on Scratchpad at normal1127×1224, QA tabs closed, viewport overrides reset. See apps/desktop/design-qa.md latest section and docs/design/compact-preview/document-screenshots.
