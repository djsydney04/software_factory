# Connect GitHub or GitLab

Use the native app to connect the registered project's `origin` repository, read open pull or merge requests, and inspect reported CI. **Settings → Integrations** owns the connection; **Reviews → Repository** owns provider reads. The browser design preview has an unavailable notice and makes no repository requests.

This integration is read only. It does not create requests or comments, approve or merge changes, push commits, deploy, or grant agents access. Local Reviews keep their own exact-revision evidence and decisions. Consult the [runtime checkpoint](runtime/IMPLEMENTATION.md) for recorded verification; a saved connection or passing build alone is not a live-provider test.

## Prepare the project and account

Start the native app and daemon as described in [getting started](getting-started.md), and register a local Git repository. Its local configuration must contain exactly one `origin` URL identifying a repository on `github.com` or `gitlab.com`. HTTPS and Git SSH formats are supported, including GitLab subgroup paths. URLs containing credentials, query parameters, fragments, encoded paths or unsupported ports are rejected. Self-hosted providers are later work.

Install the matching provider CLI separately and make it available on the daemon's `PATH` or in `~/.local/bin`. Use an account that can read the repository and its CI. If it is not already authenticated, run the appropriate supported login outside Software Factory:

```sh
gh auth login --hostname github.com
```

Or, for GitLab:

```sh
glab auth login --hostname gitlab.com
```

GitLab requires saved host-specific `glab auth login --hostname gitlab.com` authentication. Generic GitLab/CI token environment variables are deliberately excluded from its child process, so a token intended for another host cannot be sent to GitLab.com.

The app never runs interactive authentication, reads a token into the UI, or stores a provider token in its database. Credentials remain managed by the matching CLI. Do not paste them into Conversation, repository files or settings. If you install a CLI after starting the daemon, restart the daemon so executable discovery can find it, then reconnect the native client.

## Connect and inspect changes

1. Open **Settings → Integrations**. Detection reads the local `origin` and checks for the matching CLI; detection itself makes no account request.
2. Check the displayed provider and repository path, then choose **Connect GitHub** or **Connect GitLab**. Connecting performs a repository read through your existing CLI authentication before saving the project link. A failed check does not save a successful connection.
3. Open **Reviews**, choose **Repository**, and choose **Load pull requests** or **Load merge requests**. Lists contain up to 20 open requests per page. Use Previous/Next or Refresh deliberately; there is no background polling.
4. Select a request to read its current details and CI. Check the full reported head revision, fetched time and limitations. Use Refresh to read it again; provider data may have changed since the displayed snapshot.
5. Choose **Open on GitHub/GitLab** or **Open check** to inspect an allowed provider link in the system browser on macOS. Remote HTML never opens inside the trusted app window. Links on other hosts are omitted.

**Reviews → Local** remains the supervisor's local review inbox. Reading a remote request never creates a local Run or Review, applies local feedback to the provider, or changes work delivery state.

## Read CI accurately

For GitHub, selected-request details read check runs and commit statuses for the reported PR head SHA, with a 100-item limit for each source. For GitLab, details show the reported head pipeline and its revision. CI is fetched only for the selected request, not for every list row.

Missing, denied, incomplete, unknown or mismatched results are explicit. They cannot become passing evidence. A GitLab merged-results pipeline may report a different SHA from the merge-request head; that result does not certify the displayed head. A displayed success never establishes required-check policy, merge readiness, a tested merged result, or the validity of a separate local Review.

## Disconnect and diagnose

**Settings → Integrations → Disconnect** removes the saved project link. It does not log the CLI out or revoke the provider account's authentication. Reconnect to verify access again after changing `origin` or switching provider accounts.

| What you see | Next action |
| --- | --- |
| No supported repository detected | Check that the registered repository has exactly one supported local `origin` URL. |
| CLI unavailable | Install the matching CLI outside the app, ensure the daemon can discover it, then restart the daemon and detect again. |
| Authentication or access error | Check the provider CLI's supported login and the account's repository/CI permissions outside the app, then retry. No raw CLI stderr or credentials are shown. |
| Rate limit, timeout or refresh already in progress | Wait for the current read or provider limit, then refresh manually. |
| Incomplete CI or another revision | Read the limitation and compare SHAs. Refresh or inspect the provider; do not treat missing evidence as success. |
| Local service disconnected | Restore the daemon/client connection before managing the repository link or refreshing data. |

These explicit human-requested reads do not change **Settings → Permissions**. Agent execution still narrows Network and Integrations to Deny in this alpha; the connection exposes no repository API or token to the lead or worker. Existing provider sandbox limits still permit host reads, as described in [getting started](getting-started.md); connecting is not a credential-isolation boundary.
