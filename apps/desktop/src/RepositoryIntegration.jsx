import { useEffect, useReducer, useRef, useState } from "react";
import { ArrowLeft, ArrowSquareOut, ArrowsClockwise, CaretLeft, CaretRight, GitPullRequest, LinkSimple } from "@phosphor-icons/react";
import { initialRepositoryRead, repositoryCiEvidence, repositoryCiLabel, repositoryConnectionKey, repositoryFetchedLabel, repositoryProvider, repositoryReadReducer, safeRepositoryUrl } from "./repository-state.js";
import "./repository.css";

const errorText = (error) => typeof error === "string" ? error : error?.message || "The repository request failed.";

function useRepositoryRead(invoke, command, args, enabled, refresh) {
  const [state, dispatch] = useReducer(repositoryReadReducer, undefined, initialRepositoryRead);
  const generation = useRef(0);
  const scope = JSON.stringify([command, args, refresh]);
  useEffect(() => {
    if (!enabled) { dispatch({ type: "clear" }); return; }
    let active = true;
    const requestId = ++generation.current;
    dispatch({ type: "start", requestId, scope });
    invoke(command, args).then((value) => {
      if (active) dispatch({ type: "resolved", requestId, scope, value });
    }, (problem) => {
      if (active) dispatch({ type: "rejected", requestId, scope, error: errorText(problem) });
    });
    return () => { active = false; };
  }, [invoke, scope, enabled]);
  return enabled && state.scope === scope ? state : { ...initialRepositoryRead(), status: enabled ? "loading" : "idle" };
}

export function RepositoryPreviewNotice() {
  return <section className="repo-settings" aria-labelledby="repo-preview-title"><h2 id="repo-preview-title" className="section-title">Repository integrations</h2>
    <p className="muted">Connect GitHub or GitLab in the native app to read open pull or merge requests and CI status.</p>
    <p className="help">This browser is a design preview. It has no account connection and makes no repository requests.</p></section>;
}

export function RepositorySettings({ project, connection, invoke, action, busy, connected, notice }) {
  const [refresh, setRefresh] = useState(0);
  const [mutationError, setMutationError] = useState("");
  const detected = useRepositoryRead(invoke, "factory_repository_detect", { projectId: project.id }, connected && !connection, refresh);
  const candidate = detected.value?.repository;
  const provider = repositoryProvider(connection?.provider ?? candidate?.provider);
  const available = !!candidate && detected.value?.available === true;
  const mutate = async (command) => {
    setMutationError("");
    try { await action(command); }
    catch (error) { setMutationError(errorText(error)); }
  };
  return <section className="repo-settings" aria-labelledby="repo-settings-title"><div className="repo-heading"><h2 id="repo-settings-title" className="section-title">Repository integrations</h2><span className="repo-read-only">Read only</span></div>
    <p className="muted">Read open pull or merge requests and CI for this project. Agent integration permissions remain separate.</p>
    {connection ? <><div className="repo-connection"><LinkSimple size={17} aria-hidden="true" /><div><strong>{provider}</strong><p className="mono">{connection.path}</p><p className="help">{connection.host} · {connection.remoteName} · connected {repositoryFetchedLabel(connection.connectedAt)}</p></div></div>
      <div className="repo-actions"><a className="btn" href="#/reviews/repository">Open repository reviews</a><button className="btn" type="button" disabled={busy || !connected} onClick={() => mutate({ type: "disconnect_repository", projectId: project.id })}>Disconnect</button></div>
      <p className="help">Disconnect removes the saved project link. CLI authentication stays on this computer.</p></>
      : <><div className="repo-connection"><GitPullRequest size={17} aria-hidden="true" /><div><strong>{detected.status === "loading" ? "Inspecting origin…" : candidate ? provider : "No supported repository detected"}</strong>
        {candidate && <p className="mono">{candidate.path}</p>}<p className="help">{detected.status === "loading" ? "Reading the local Git remote and installed CLI. No account request is made." : detected.value?.reason || (candidate ? `Detected from origin on ${candidate.host}.` : "Use an origin remote on github.com or gitlab.com.")}</p></div></div>
        {detected.status === "error" && <p className="error" role="alert">{detected.error}</p>}
        <div className="repo-actions"><button type="button" className="btn primary" disabled={busy || !connected || !available || detected.status !== "ready"} onClick={() => mutate({ type: "connect_repository", projectId: project.id, provider: candidate.provider })}>Connect {candidate ? provider : "repository"}</button>
          <button type="button" className="btn" disabled={!connected || detected.status === "loading"} onClick={() => setRefresh((value) => value + 1)}><ArrowsClockwise size={14} aria-hidden="true" /> Detect again</button></div>
        <p className="help">Connecting verifies access with your existing CLI account and saves this repository link. It does not import credentials or grant agents access.</p></>}
    <details className="repo-auth"><summary>Authentication setup</summary><p className="help">Authenticate the matching CLI outside the app, then detect again or retry the read. Credentials stay with that CLI.</p><p>GitHub <code>gh auth login --hostname github.com</code></p><p>GitLab <code>glab auth login --hostname gitlab.com</code></p><p className="help">The matching CLI must be installed. Self-hosted providers are not supported yet.</p></details>
    {(mutationError || notice) && <p className={mutationError ? "error" : "notice"} role={mutationError ? "alert" : "status"}>{mutationError || notice}</p>}
    {!connected && <p className="error" role="status">Reconnect to the local service to manage this connection.</p>}
  </section>;
}

function ProviderOpen({ url, connection, invoke, children = "Open on provider" }) {
  const [error, setError] = useState("");
  const safe = safeRepositoryUrl(url, connection);
  if (!safe) return null;
  return <span className="repo-open"><button type="button" className="text-link" onClick={async () => { setError(""); try { await invoke("factory_open_repository_url", { url: safe }); } catch (problem) { setError(errorText(problem)); } }}>{children} <ArrowSquareOut size={13} aria-hidden="true" /></button>{error && <span className="error small" role="alert">{error}</span>}</span>;
}

export function RepositoryReviews({ project, connection, invoke, connected }) {
  if (!connection) return <section className="repo-review-empty"><LinkSimple size={22} aria-hidden="true" /><h1 className="pane-title">Connect a repository</h1><p className="muted">See open GitHub pull requests or GitLab merge requests and their CI status here.</p><a className="btn" href="#/settings/integrations">Set up integration</a></section>;
  return <ConnectedRepositoryReviews key={repositoryConnectionKey(connection)} project={project} connection={connection} invoke={invoke} connected={connected} />;
}

function ConnectedRepositoryReviews({ project, connection, invoke, connected }) {
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState(null);
  const [refresh, setRefresh] = useState(0);
  const [requested, setRequested] = useState(false);
  const heading = useRef(null);
  const rows = useRef({});
  const returnTo = useRef(null);
  const list = useRepositoryRead(invoke, "factory_repository_changes", { projectId: project.id, page }, connected && requested && selected == null, refresh);
  const detail = useRepositoryRead(invoke, "factory_repository_change", { projectId: project.id, number: selected }, connected && selected != null, refresh);
  const provider = repositoryProvider(connection.provider);
  const noun = connection.provider === "gitlab" ? "merge requests" : "pull requests";
  const current = selected == null ? list : detail;
  const load = () => { setRequested(true); setRefresh((value) => value + 1); };
  useEffect(() => { if (selected != null) heading.current?.focus(); }, [selected]);
  useEffect(() => {
    if (selected == null && list.status === "ready" && returnTo.current != null) {
      (rows.current[returnTo.current] ?? heading.current)?.focus(); returnTo.current = null;
    }
  }, [selected, list.status, list.value]);
  const back = () => { returnTo.current = selected; setSelected(null); };
  const result = list.value;
  const evidence = detail.value ? repositoryCiEvidence(detail.value) : null;
  return <section className={`repo-reviews ${selected != null ? "repo-selected" : ""}`} aria-labelledby="repo-reviews-title"><header className="repo-reviews-head">
    {selected != null && <button type="button" className="text-link repo-back" onClick={back}><ArrowLeft size={14} aria-hidden="true" /> {provider} {noun}</button>}
    <div className="repo-heading"><div><h1 className="pane-title" id="repo-reviews-title" tabIndex={-1} ref={heading}>{selected != null ? `${connection.provider === "gitlab" ? "Merge" : "Pull"} request #${selected}` : `${provider} ${noun}`}</h1><p className="mono small muted">{connection.path}</p></div>
      <button type="button" className="btn sm" disabled={!connected || current.status === "loading"} onClick={load}><ArrowsClockwise size={14} aria-hidden="true" />{requested || selected != null ? "Refresh" : `Load ${noun}`}</button></div>
    <p className="help">Read-only provider snapshot. Local review decisions and run evidence are separate.</p>
    {current.value?.fetchedAt && <p className="repo-fetched small muted">Fetched {repositoryFetchedLabel(current.value.fetchedAt)} · refresh to check for newer changes.</p>}
  </header>
    {!connected && <div className="repo-message" role="status"><h2>Local service disconnected</h2><p className="muted">Reconnect before reading provider data.</p></div>}
    {current.status === "loading" && <p className="repo-message muted" role="status">{selected == null ? `Loading ${noun}…` : "Reading request and CI status…"}</p>}
    {current.status === "error" && <div className="repo-message" role="alert"><h2>Repository read failed</h2><p className="error">{current.error}</p><p className="help">Check CLI authentication in Settings → Integrations, then refresh. No request was changed.</p><a className="text-link" href="#/settings/integrations">Integration settings</a></div>}
    {selected == null && current.status === "idle" && connected && <div className="repo-message"><h2>Ready when you are</h2><p className="muted">Load open {noun} from {provider}. Reads happen only when you open a request, change page, or refresh.</p></div>}
    {selected == null && result && <><div className="repo-list-scroll">{result.items.length ? <ul className="repo-change-list" aria-label={`Open ${noun}`}>{result.items.map((item) => <li key={item.number}><button className="repo-change-row" type="button" ref={(element) => { rows.current[item.number] = element; }} onClick={() => setSelected(item.number)}><GitPullRequest size={17} aria-hidden="true" /><span className="repo-row-copy"><span className="repo-row-title">{item.title}</span><span className="repo-row-meta"><span className="mono">#{item.number}</span><span>{item.author || "Author unavailable"}</span>{item.draft && <span>Draft</span>}<span>{item.sourceBranch} → {item.targetBranch}</span></span></span><CaretRight size={16} aria-hidden="true" /></button></li>)}</ul> : <div className="repo-message"><h2>{page === 1 ? `No open ${noun}` : "No more results on this page"}</h2><p className="muted">{page === 1 ? `The connected repository returned no open ${noun}.` : "Return to the previous page or refresh."}</p></div>}</div>
      <nav className="repo-pagination" aria-label="Repository request pages"><button className="btn sm" type="button" disabled={page === 1 || current.status === "loading"} onClick={() => setPage((value) => value - 1)}><CaretLeft size={14} aria-hidden="true" /> Previous</button><span className="small muted">Page {page} · up to 20 per page</span><button className="btn sm" type="button" disabled={!result.hasMore || current.status === "loading"} onClick={() => setPage((value) => value + 1)}>Next <CaretRight size={14} aria-hidden="true" /></button></nav></>}
    {selected != null && detail.value && <div className="repo-detail-scroll"><article className="repo-detail"><header><div className="repo-detail-title"><h2>{detail.value.change.title}</h2>{detail.value.change.draft && <span className="repo-read-only">Draft</span>}</div><p className="small muted">{detail.value.change.author || "Author unavailable"} · {detail.value.change.sourceBranch} → {detail.value.change.targetBranch}</p><p className="repo-revision small"><span className="muted">Head revision</span><code>{detail.value.change.headSha || "Not reported"}</code></p><ProviderOpen url={detail.value.change.url} connection={connection} invoke={invoke}>Open on {provider}</ProviderOpen></header>
      <section className="repo-ci" aria-labelledby="repo-ci-title"><div className="repo-heading"><h3 id="repo-ci-title">CI status</h3><span className={`repo-ci-state repo-ci-${evidence.passed ? "success" : evidence.state}`}>{evidence.label}</span></div>
        {!evidence.current && <p className="repo-ci-note warn-text">CI belongs to a different or unknown revision and does not certify this head. Refresh to read it again.</p>}
        {!evidence.complete && <p className="repo-ci-note warn-text">The provider response is incomplete. Missing results are not passing checks.</p>}
        {evidence.items.length > 0 ? <ul className="repo-check-list" aria-label="Provider CI checks">{evidence.items.map((item, index) => {
          const stale = !evidence.current || !!item.revision && item.revision !== evidence.revision;
          const success = item.state === "success" && !stale && evidence.complete;
          return <li key={`${item.name}-${index}`}><div><strong>{item.name}</strong><p className="small muted">{item.rawStatus || "Status not reported"}{stale ? " · older or unknown revision" : ""}</p>{item.revision && <p className="mono small muted">{item.revision}</p>}<ProviderOpen url={item.url} connection={connection} invoke={invoke}>Open check</ProviderOpen></div><span className={`repo-ci-state repo-ci-${success ? "success" : stale ? "unknown" : item.state === "success" ? "unknown" : item.state}`}>{stale ? "Not current" : item.state === "success" && !evidence.complete ? "Unverified" : repositoryCiLabel(item.state)}</span></li>;
        })}</ul> : <p className="muted repo-ci-note">No CI checks were reported. This is not passing evidence.</p>}
        {evidence.limitations.length > 0 && <ul className="repo-ci-limitations">{evidence.limitations.map((item, index) => <li key={index}>{item}</li>)}</ul>}
        <p className="help">Provider CI is informational. It never accepts a local review, verifies a different SHA, or merges work.</p>
      </section></article></div>}
  </section>;
}
