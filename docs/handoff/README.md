# Implementation handoff

Start at [START-HERE.md](../../START-HERE.md). Paste [AGENT-PROMPT.md](AGENT-PROMPT.md) into the implementing agent, with the complete repository available.

- [Build plan](BUILD-PLAN.md): current implementation truth, amendments to earlier proposals, and the first usable local workflow.
- [Design index](DESIGN-INDEX.md): current reference screens and the complete mock/history inventory.
- [Original source](source/README.md): user objective and five supplied visual references, with portable paths.
- [Historical archive](archive/README.md): public final plans, prompts, checkpoints, earlier contracts and reviews.
- [Current QA](../../apps/desktop/design-qa.md): executed frontend checks and their limits; later sections supersede earlier counts/layouts.
- [Handoff verification](VERIFICATION.md): clean source-copy install/build/tests, development-server checks, and package integrity checks.
- [Snapshot manifest](manifest.json): included relative paths, byte counts and SHA-256 hashes at packaging time, plus recorded Git base and exclusions. This is an inventory of the handoff, not a runtime capability claim. The manifest excludes itself to avoid a circular hash.

The portable ZIP includes current source, tests, manifests/lockfiles, public frontend assets, all repository design mocks and screenshots, current decisions/contracts, architectural research, and the additional source/archive files above. It includes untracked project files. Git metadata/history, dependencies, build output, `.env` files, credentials, browser-local state, raw agent streams, Codex session databases and temporary QA fixtures are excluded. Install dependencies and build using START-HERE.

The recorded Git SHA describes the base checkout; it is **not** a commit containing this full handoff. No source commit, push, deployment or continuously running agent is implied by the archive. The local-alpha execution evidence is recorded separately in [the real trial](../runtime/INTEGRATION-TRIAL.md). Preserve that distinction when continuing from another machine or a fresh agent environment.
