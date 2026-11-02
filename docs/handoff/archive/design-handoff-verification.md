# Handoff verification

Verified November 1, 2026. This records the source/design handoff's checks, not a claim that a factory runtime has been implemented.

## Clean source-copy check

A separate copy was created with the packaged source selection and **without** existing dependencies, build output, or Git metadata. Using Node 24.22.1 and pnpm 11.25.0, these commands ran in its `apps/desktop` directory:

```sh
pnpm install --frozen-lockfile
pnpm build
node --test tests/*.test.mjs
pnpm dev --host 127.0.0.1 --port 5187 --strictPort
```

- Install succeeded with 66 packages, reusing the host's pnpm cache. This was a clean application directory on the same host, not a fresh operating-system installation.
- Production build succeeded and produced the documented Sites-compatible client/server/hosting files.
- All **45 tests** passed: 41 domain tests and four Sites packaging tests. No tests were skipped.
- The separate development server served `/`, `/src/main.jsx`, and `/assets/greenroom-collection.png` with HTTP 200 and nonempty expected content. It was then stopped. The user's existing port-5173 preview was left running.
- The tested app files were compared byte-for-byte with the current handoff source before packaging. Later handoff documentation edits did not change that tested application.

## Package and documentation checks

All local Markdown/image links are checked against files within the handoff, including historical public briefs whose source links were made portable. Original reference PNG hashes match the supplied files; the original objective differs only in its five attachment paths. Older design notes remain labelled history.

Packaging verifies ZIP integrity and every included file's size and SHA-256 against [manifest.json](design-handoff-manifest.json). The manifest includes current untracked source; its recorded Git SHA is the base commit, not a commit containing the handoff. The manifest excludes itself from its own hash inventory.

The ZIP contains no Git metadata, installed dependencies, build cache/output, environment files, browser-local state, raw agent streams, provider account/session databases, or temporary QA source fixtures. Existing curated provenance and labelled UI evidence remain included.

## Limits

The [desktop QA record](../../../apps/desktop/design-qa.md) owns prior interactive, responsive, scrolling and permission-snapshot checks. No new UI behavior was introduced by this documentation handoff. No real factory provider run, Rust build, native app, TUI, permission enforcement, recurring job, deployment, push, or GitHub merge was performed. The [next agent's build plan](../BUILD-PLAN.md) covers those implementation gaps and requires actual evidence before claiming usability.
