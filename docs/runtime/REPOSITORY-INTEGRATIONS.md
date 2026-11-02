# Repository integration verification

The follow-up scope is human-requested, read-only GitHub/GitLab connections, open pull/merge requests and reported CI. The initial native alpha was committed and pushed as `8e3fbd7`, preserving four newer remote design/skill commits. [Setup and usage](../integrations.md) describes this new capability; the original [local-alpha trial](INTEGRATION-TRIAL.md), goal and frozen handoff inventories remain historical evidence.

## Implemented

- Schema 3 adds project repository connections with transactional events, idempotent connect/disconnect, and schema-2 migration. No provider credentials are stored.
- Native Settings → Integrations detects exactly one local `origin`, verifies access and saves its canonical GitHub.com or GitLab.com identity. GitLab subgroup paths are supported. Unsupported hosts and credential-bearing URLs fail visibly.
- Reviews → Repository loads 20 open requests per page, then reads CI only for the selected request. Refresh/pagination are deliberate reads; normal two-second daemon snapshots do not poll providers.
- GitHub details combine check runs and legacy commit statuses. GitLab details expose the head pipeline and its SHA. Missing, incomplete, unknown and mismatched CI cannot become passing evidence. Remote status never accepts a local Review or establishes merge readiness.
- Existing `gh` or saved host-specific `glab` authentication stays outside the UI. Fixed GET methods and absolute public API URLs prevent CLI host configuration from redirecting requests. GitLab child processes exclude generic token/refresh/CI variables rather than re-scoping a token for another host.
- A trusted native command opens allowlisted provider links in the macOS system browser. Existing navigation/CSP boundaries keep remote HTML outside the application. Agent Network/Integrations ceilings remain unchanged.

## Executed checks

The full Rust suite passed **75 tests**, including eight new integration tests. After the final GitLab authentication-environment change, all eight integration tests passed again with deliberately fake inherited token aliases, proving they did not reach the GitLab fixture. Frontend build and all **61 tests** passed, including stale-response, exact-SHA CI and provider-link cases. Both native security tests passed, and the final unsigned debug `.app` bundled successfully. Root and native Rust formatting checks passed.

The native application was exercised with a deterministic external CLI fixture: connect, bounded list, selected detail, a failed legacy check beside a passing check, refresh to missing/partial CI, disconnect, and GitLab's missing-CLI state. Desktop and 394px layouts were inspected. Eight fixture calls were fixed GETs in the neutral daemon directory; no debug environment reached them, no background API calls appeared, and no model Request/Task/Run was created. The screenshots below are **fixture evidence**, not provider results:

- [Desktop failed CI](evidence/repository-integration/repository-failed-desktop.png)
- [Phone CI detail](evidence/repository-integration/repository-failed-phone.png)
- [Partial CI replaces prior checks](evidence/repository-integration/repository-partial-desktop.png)
- [GitLab requires its CLI](evidence/repository-integration/gitlab-missing-cli-desktop.png)
- [Fixture result](evidence/repository-integration/ui-result.json)

The rebuilt native app then connected to the **real GitHub** repository `djsydney04/software_factory` using installed `gh 2.102.0` and existing supported authentication. Its actual repository read succeeded and the open-PR list returned zero items. That empty state was rendered honestly. No real selected-PR CI result exists for this repository because it had no open PR; the adapters' CI cases were verified with fixtures.

- [Live native GitHub list](evidence/repository-integration/github-live-reviews.png)
- [Sanitized live response](evidence/repository-integration/github-live-result.json)

GitLab live access was **not** verified: `glab` is not installed or authenticated on this machine. Its adapter, subgroup encoding, pipeline revision mismatch, missing authentication and API bounds were exercised deterministically. Follow [GitLab setup](../integrations.md) before using it.

No provider writes, PR/MR creation, approvals, merge, deployment, new model inference or recurring-job installation occurred. Source commits/pushes are the user's separately authorized repository publication. Private fixture state and native connection credentials stay outside the checkout. The final app is launched against the default user state, ready to connect a project; the paused integration fixture is retained separately.
