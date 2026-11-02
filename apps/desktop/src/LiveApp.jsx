import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, List, Pause, Play } from "@phosphor-icons/react";
import { Conversation, Drawer, EMPTY_COMPOSER, parseHash, Sidebar } from "./App.jsx";
import { RunsPage } from "./Runs.jsx";
import { ProgressPage } from "./Progress.jsx";
import { ReviewsPage } from "./Reviews.jsx";
import { ScratchpadPage, ScratchpadSettings } from "./Scratchpad.jsx";
import { restoreScratchpad } from "./scratchpad.js";
import { ModelSelect, EffortSelect, Notice, Tabs } from "./ui.jsx";
import { PermissionFields, PermissionFacts } from "./Permissions.jsx";
import { resolvePermissions } from "./permissions.js";
import { validateTiming } from "./scheduling.js";
import { ACTIVE } from "./data.js";
import { buildLiveView, emptySnapshot, liveDefaults, liveScheduleLabel, observedCatalog, validateLiveCombo, validateSnapshot } from "./live-state.js";
import "./live.css";

const UI_KEY = "software-factory.native-ui.v1";
const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);
const errorText = (error) => typeof error === "string" ? error : error?.message || "The local service request failed.";
function readUi() {
  try {
    const value = JSON.parse(localStorage.getItem(UI_KEY) || "null");
    return { ...(value && typeof value === "object" ? value : {}), readError: false };
  } catch { return { readError: true }; }
}

function useFactory(ui, setUi) {
  const [snapshot, setSnapshot] = useState(emptySnapshot);
  const [capabilities, setCapabilities] = useState(null);
  const [connection, setConnection] = useState("loading");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(ui.pendingCommand ?? null);
  const inFlight = useRef(null);
  const refresh = useCallback(async () => {
    const value = validateSnapshot(await invoke("factory_snapshot"));
    setSnapshot((previous) => value.lastEventSequence >= previous.lastEventSequence ? value : previous);
    setConnection("connected"); setError("");
    try { setCapabilities(await invoke("factory_capabilities")); }
    catch (problem) { setCapabilities({ available: false, limitations: [errorText(problem)] }); }
    return value;
  }, []);
  useEffect(() => {
    let stopped = false, timer;
    const poll = async () => {
      try { await refresh(); }
      catch (problem) { if (!stopped) { setConnection("disconnected"); setError(errorText(problem)); } }
      if (!stopped) timer = window.setTimeout(poll, 2000);
    };
    poll();
    return () => { stopped = true; window.clearTimeout(timer); };
  }, [refresh]);
  const execute = useCallback(async (command) => {
    const signature = JSON.stringify(command);
    if (inFlight.current) {
      if (JSON.stringify(pending.current?.command) !== signature) throw new Error("Wait for the current command to finish.");
      return inFlight.current;
    }
    if (pending.current && JSON.stringify(pending.current.command) !== signature) throw new Error("A previous command needs reconciliation. Retry that saved action first.");
    const envelope = pending.current ?? { commandId: crypto.randomUUID(), command };
    pending.current = envelope;
    // Save identity before transport so a lost response can be retried without duplicate execution.
    try {
      const current = readUi();
      localStorage.setItem(UI_KEY, JSON.stringify({ ...current, pendingCommand: envelope }));
    } catch { /* The UI exposes its storage warning; this window retains the identity. */ }
    setUi((previous) => ({ ...previous, pendingCommand: envelope }));
    setBusy(true);
    const operation = (async () => {
      try {
        const result = await invoke("factory_command", { command: envelope });
        pending.current = null;
        setUi((previous) => ({ ...previous, pendingCommand: null }));
        try { await refresh(); }
        catch (problem) { setConnection("disconnected"); setError(`Command accepted; reconnect to inspect the saved result. ${errorText(problem)}`); }
        return result;
      } catch (problem) {
        const message = errorText(problem);
        if (message.startsWith("Local service rejected the request:")) {
          pending.current = null;
          setUi((previous) => ({ ...previous, pendingCommand: null }));
        }
        throw new Error(message);
      } finally { inFlight.current = null; setBusy(false); }
    })();
    inFlight.current = operation;
    return operation;
  }, [refresh, setUi]);
  return { snapshot, capabilities, connection, error, busy, execute, refresh };
}

export function LiveApp() {
  const [ui, setUi] = useState(readUi);
  const [storeError, setStoreError] = useState(ui.readError);
  const [route, setRoute] = useState(parseHash);
  const [drawer, setDrawer] = useState(false);
  const [notice, setNotice] = useState("");
  const [focusComposer, setFocusComposer] = useState(0);
  const [now, setNow] = useState(Date.now);
  const factory = useFactory(ui, setUi);
  const { project, runs, tasks, reviews, requests, messages, reviewState } = useMemo(() => buildLiveView(factory.snapshot, ui.projectId), [factory.snapshot, ui.projectId]);
  const catalog = useMemo(() => observedCatalog(factory.capabilities), [factory.capabilities]);
  const composer = { ...EMPTY_COMPOSER, ...(ui.composer ?? {}), override: { ...EMPTY_COMPOSER.override, ...(ui.composer?.override ?? {}) }, attached: [] };
  const setComposer = (update) => setUi((previous) => ({ ...previous, composer: typeof update === "function" ? update({ ...EMPTY_COMPOSER, ...previous.composer, override: { ...EMPTY_COMPOSER.override, ...previous.composer?.override }, attached: [] }) : update }));
  const scratchpad = useMemo(() => restoreScratchpad(ui.scratchpad), [ui.scratchpad]);
  const setScratchpad = (update) => setUi((previous) => ({ ...previous, scratchpad: typeof update === "function" ? update(restoreScratchpad(previous.scratchpad)) : update }));
  const setReviewState = (update) => setUi((previous) => {
    const value = { ...reviewState, drafts: previous.reviewDrafts ?? {} };
    const next = typeof update === "function" ? update(value) : update;
    return { ...previous, reviewDrafts: next.drafts };
  });
  const runsUi = { filter: "all", query: "", view: "list", tab: "activity", ...(ui.runsUi ?? {}) };
  const setRunsUi = (update) => setUi((previous) => ({ ...previous, runsUi: typeof update === "function" ? update({ filter: "all", query: "", view: "list", tab: "activity", ...previous.runsUi }) : update }));
  const reviewsUi = { filter: "all", query: "", views: {}, ...(ui.reviewsUi ?? {}) };
  const setReviewsUi = (update) => setUi((previous) => ({ ...previous, reviewsUi: typeof update === "function" ? update({ filter: "all", query: "", views: {}, ...previous.reviewsUi }) : update }));
  useEffect(() => {
    const changed = () => { setRoute(parseHash()); setNotice(""); };
    const timer = window.setInterval(() => setNow(Date.now()), 30_000);
    window.addEventListener("hashchange", changed);
    return () => { window.removeEventListener("hashchange", changed); window.clearInterval(timer); };
  }, []);
  useEffect(() => {
    try { localStorage.setItem(UI_KEY, JSON.stringify(ui)); setStoreError(false); }
    catch { setStoreError(true); }
  }, [ui]);
  const navigate = (path) => { window.location.hash = `#/${path}`; };
  const action = async (command, success = "") => {
    setNotice("");
    try { const result = await factory.execute(command); if (success) setNotice(success); return result; }
    catch (problem) { setNotice(errorText(problem)); throw problem; }
  };
  const safeAction = (command, success) => { action(command, success).catch(() => {}); };
  const retryPending = async () => {
    const command = ui.pendingCommand?.command;
    if (!command) return;
    try {
      await action(command);
      const sentText = command.type === "send_steering" ? command.text : command.type === "submit_request" ? command.request.text : null;
      if (sentText && sentText === composer.draft.trim()) { setComposer(structuredClone(EMPTY_COMPOSER)); setFocusComposer((value) => value + 1); }
    } catch { /* Retain this same identity until the outcome can be reconciled. */ }
  };
  const settings = project ? { ...project.settings, projectName: project.name, mode: "bounded" } : null;
  const nextRun = settings ? { ...settings, model: composer.override.model ?? settings.model, effort: composer.override.effort ?? settings.effort,
    ...resolvePermissions(settings, composer.permissionOverride) } : null;
  const submit = async () => {
    if (!project || factory.busy || !composer.draft.trim()) return;
    let command;
    if (composer.mode === "oneoff") {
      const invalid = validateLiveCombo(catalog, nextRun.model, nextRun.effort);
      if (invalid) { setNotice(invalid); return; }
      const timing = validateTiming(composer.policy, composer.localTime);
      if (timing.error) { setNotice(timing.error); return; }
      const { projectName, mode, ...captured } = nextRun;
      command = { type: "submit_request", request: { projectId: project.id, text: composer.draft.trim(),
        policy: { lead: "lead_decides", next: "next_available", at: "scheduled" }[composer.policy],
        scheduledAt: composer.policy === "at" ? new Date(timing.scheduledAt).toISOString() : null,
        timeZone: timing.timeZone, settings: captured, source: "conversation" } };
    } else command = { type: "send_steering", projectId: project.id, text: composer.draft.trim() };
    try { await action(command); setComposer(structuredClone(EMPTY_COMPOSER)); setFocusComposer((value) => value + 1); }
    catch { /* Keep the draft and pending command identity for recovery. */ }
  };
  const decide = (runId, decision) => {
    const approval = factory.snapshot.approvals.find((item) => item.runId === runId && item.decision == null);
    if (approval) safeAction({ type: "resolve_approval", approvalId: approval.id, allow: decision === "allow" }, "Approval decision recorded by the service.");
  };
  const page = route.page;
  const newOneOff = () => { setComposer((previous) => ({ ...previous, mode: "oneoff" })); navigate("conversation"); setFocusComposer((value) => value + 1); };
  const view = route.schedule ? "schedule" : runsUi.view;
  const progressView = ui.progressView ?? "graph";
  const running = runs.filter((run) => ACTIVE.includes(run.status));
  const leadWorking = requests.some((request)=>request.status==="planning") || factory.snapshot.leadWakes.some((wake)=>["prepared","running"].includes(wake.status));
  const leadRecovery = requests.some((request)=>request.status==="recovery_required") || factory.snapshot.leadWakes.some((wake)=>wake.status==="recovery_required");
  const waiting = running.length ? null : requests.findLast((request) => ["waiting", "recovery_required"].includes(request.status) && request.reason);
  const status = !project ? "Local alpha" : factory.connection !== "connected" ? "Disconnected" : project.paused ? "Paused" : leadRecovery ? "Needs reconciliation" : running.length ? `${running.length} active` : leadWorking ? "Lead working" : waiting ? "Waiting" : "Ready";
  const sidebar = <Sidebar page={page} readyReviews={reviews.filter((review) => review.group === "ready").length} projectName={project?.name ?? "Software Factory"} onNavigate={() => setDrawer(false)} live />;
  const switcher = page === "progress" ? <Tabs id="progview" label="Progress view" value={progressView} onChange={(value) => setUi((previous) => ({ ...previous, progressView: value }))} items={[{ id: "graph", label: "Graph" }, { id: "board", label: "Board" }, { id: "radar", label: "Radar" }]} />
    : page === "runs" && !route.id ? <Tabs id="runsview" label="Runs view" value={view} onChange={(value) => { setRunsUi((previous) => ({ ...previous, view: value })); navigate(value === "schedule" ? "runs/schedule" : "runs"); }} items={[{ id: "list", label: "List" }, { id: "timeline", label: "Timeline" }, { id: "schedule", label: "Schedule" }]} /> : null;
  return <div className="app live-app"><aside className="sidebar desktop-only" aria-label="Primary">{sidebar}</aside><Drawer open={drawer} onClose={() => setDrawer(false)}>{sidebar}</Drawer>
    <div className="main"><header className={`topbar ${switcher ? "has-switch" : ""}`}><button type="button" className="icon-btn menu-btn" aria-label="Open navigation" aria-expanded={drawer} onClick={() => setDrawer(true)}><List size={18} /></button>
      <p className="goal"><span className="goal-title">{project?.objective ?? "Connect a repository and goal"}</span>{waiting && <span className="goal-focus"> · {waiting.reason}</span>}</p>
      <div className="topbar-right">{switcher}<span className="run-state"><span className="dot" aria-hidden="true" />{status}</span>{project && <button type="button" className="icon-btn" disabled={factory.busy || factory.connection !== "connected"} aria-label={project.paused ? "Resume future dispatch" : "Pause future dispatch"} title={project.paused ? "Resume future dispatch" : "Pause future dispatch; active attempts continue"} onClick={() => safeAction({ type: "set_paused", projectId: project.id, paused: !project.paused })}>{project.paused ? <Play size={15} /> : <Pause size={15} />}</button>}<span className="preview-tag">Local alpha</span></div>
    </header>
    {(factory.error || ui.pendingCommand || storeError) && <div className="live-connection" role="status">{factory.error && <span>{factory.error}</span>}{storeError && <span>Draft storage unavailable. Keep this window open to preserve unfinished text.</span>}{ui.pendingCommand && <span>Saved action awaiting a confirmed response. <button className="text-link" disabled={factory.busy} onClick={retryPending}>Retry saved action</button></span>}{factory.connection === "disconnected" && <button className="text-link" onClick={() => factory.refresh().catch((problem) => setNotice(errorText(problem)))}>Reconnect</button>}</div>}
    <main className={`page page-${page}`} id="main">
      {!project ? factory.connection === "loading" ? <div className="empty-state"><p>Connecting to the local service…</p></div> : factory.connection === "disconnected" ? <div className="empty-state"><h1 className="pane-title">Local service unavailable</h1><p>Start the independent Factory service, then reconnect. No sample project or activity is loaded here.</p><Notice text={notice} /></div> : <RegisterProject busy={factory.busy} notice={notice} onRegister={(project) => action({ type: "register_project", project }).then((result) => { setUi((previous) => ({ ...previous, projectId: result.id })); navigate("settings/models"); })} /> : <>
        {page === "conversation" && <Conversation messages={messages} runs={runs} composer={composer} setComposer={setComposer} settings={settings} nextRun={nextRun} submit={submit} toggleAttach={() => {}} decisions={{}} decide={decide} notice={notice} openReview={(id) => navigate(`reviews/${id}`)} focusComposer={focusComposer} storeError={storeError} reviews={reviews} live={{ busy: factory.busy || factory.connection !== "connected", catalog, validate: (model, effort) => validateLiveCombo(catalog, model, effort) }} />}
        {page === "runs" && ((!route.id && view === "schedule") || route.requestId ? <LiveSchedule requests={requests} tasks={tasks} runs={runs} requestId={route.requestId} navigate={navigate} newOneOff={newOneOff} busy={factory.busy} notice={notice} cancel={(id) => safeAction({ type: "cancel_request", requestId: id })} /> : <RunsPage runs={runs} tasks={tasks} runId={route.id} ui={{ ...runsUi, view }} setUi={setRunsUi} navigate={navigate} settings={settings} retry={() => null} interrupt={(runId) => safeAction({ type: "interrupt_run", runId }, "Interrupt request saved.")} cancelQueued={() => {}} decisions={{}} decide={decide} notice={notice} openReview={(id) => navigate(`reviews/${id}`)} />)}
        {page === "progress" && <ProgressPage live view={progressView} runs={runs} tasks={tasks} reviewStatus={reviewState.status} openRun={(id) => navigate(`runs/${id}`)} decisions={{}} decide={decide} notice={notice} goal={{ title: project.objective, leadStatus: leadRecovery ? "Needs reconciliation" : leadWorking ? "Applying project context" : running.length ? "Coordinating active work" : "Waiting for eligible work" }} now={now} />}
        {page === "reviews" && <ReviewsPage reviewId={route.id} navigate={navigate} runs={runs} tasks={tasks} reviews={reviews} mergedHistory={[]} state={{ ...reviewState, drafts: ui.reviewDrafts ?? {} }} setState={setReviewState} ui={reviewsUi} setUi={setReviewsUi} openRun={(id) => navigate(`runs/${id}`)} storeError={storeError} busy={factory.busy} onDecision={(decision) => action({ type: "record_local_review_decision", reviewId: decision.reviewId, decision: decision.type === "merge" ? "accepted_local" : "changes_requested", feedback: decision.text ?? "" })} />}
        {page === "scratchpad" && <ScratchpadPage value={scratchpad} setValue={setScratchpad} requests={[]} runs={[]} storeError={storeError} />}
        {page === "files" && <div className="empty-state"><h1 className="pane-title">Files & context</h1><p className="mono">{project.repoPath}</p><p>Agents receive the captured repository objective and steering in isolated worktrees. Editing project files from this client is not connected yet.</p><a className="text-link" href="#/runs">Inspect captured attempt context <ArrowRight size={13} /></a></div>}
        {page === "settings" && <LiveSettings project={project} catalog={catalog} capabilities={factory.capabilities} section={route.id} navigate={navigate} busy={factory.busy} notice={notice} save={(settings) => action({ type: "save_settings", projectId: project.id, settings }, "Project settings saved. Previous requests and attempts retain their snapshots.")} objective={(objective) => action({ type: "set_objective", projectId: project.id, objective }, "Objective saved for future planning turns.")} scratchpad={scratchpad} setScratchpad={setScratchpad} storeError={storeError} />}
      </>}
    </main></div></div>;
}

function RegisterProject({ onRegister, busy, notice }) {
  const [draft, setDraft] = useState({ name: "", repoPath: "", objective: "" });
  const set = (key, value) => setDraft((previous) => ({ ...previous, [key]: value }));
  return <section className="live-registration"><h1 className="pane-title">Start a local project</h1><p className="muted">Connect one Git repository and give its lead a persistent goal.</p>
    <form onSubmit={(event) => { event.preventDefault(); onRegister({ ...draft, settings: liveDefaults() }).catch(() => {}); }}>
      <label className="field"><span className="field-label">Project name</span><input required value={draft.name} onChange={(event) => set("name", event.target.value)} /></label>
      <label className="field"><span className="field-label">Repository path</span><input required className="mono" placeholder="/absolute/path/to/repository" value={draft.repoPath} onChange={(event) => set("repoPath", event.target.value)} /><span className="help">An existing local Git working-tree root. Registration does not run an agent.</span></label>
      <label className="field"><span className="field-label">Persistent goal</span><textarea required rows={5} value={draft.objective} onChange={(event) => set("objective", event.target.value)} /></label>
      <Notice text={notice} /><button type="submit" className="btn primary" disabled={busy || !draft.name.trim() || !draft.repoPath.trim() || !draft.objective.trim()}>Connect project</button>
    </form></section>;
}

function LiveSettings({ project, catalog, capabilities, section, navigate, busy, notice, save, objective, scratchpad, setScratchpad, storeError }) {
  const tabs = [{ id: "general", label: "General" }, { id: "models", label: "Models & execution" }, { id: "permissions", label: "Permissions" }, { id: "files", label: "Files & context" }, { id: "scratchpad", label: "Scratchpad" }];
  const tab = tabs.some((item) => item.id === section) ? section : "models";
  const [draft, setDraft] = useState(project.settings);
  const [goal, setGoal] = useState(project.objective);
  useEffect(() => { setDraft(project.settings); }, [JSON.stringify(project.settings)]);
  useEffect(() => { setGoal(project.objective); }, [project.objective]);
  const dirty = JSON.stringify(draft) !== JSON.stringify(project.settings);
  const modelError = validateLiveCombo(catalog, draft.model, draft.effort);
  const set = (key, value) => setDraft((previous) => ({ ...previous, [key]: value }));
  const submit = (event) => { event.preventDefault(); (tab === "general" ? objective(goal) : save(draft)).catch(() => {}); };
  return <div className="settings"><div className="settings-nav"><h1 className="pane-title">Settings</h1><Tabs id="settings" label="Settings categories" vertical items={tabs} value={tab} onChange={(next) => navigate(`settings/${next}`)} /></div>
    <div className="settings-form" id="settings-panel" role="tabpanel" aria-labelledby={`settings-tab-${tab}`}><p className="scope-line"><strong>Project defaults</strong> · {project.name} · future sends and attempts.</p>
      {tab === "scratchpad" ? <><p className="help">This document and review plan are saved in this client. Recurring agent triage is a later milestone; no cron is installed.</p><ScratchpadSettings live value={scratchpad} setValue={setScratchpad} onTriage={() => {}} notice="" storeError={storeError} /></> : <form onSubmit={submit}>
      {tab === "general" && <div className="fields"><h2 className="section-title">{project.name}</h2><p className="mono live-path">{project.repoPath}</p><label className="field"><span className="field-label">Persistent goal</span><textarea rows={7} value={goal} onChange={(event) => setGoal(event.target.value)} required /></label><p className="help">Context revision {project.contextRevision}. Changing the goal does not rewrite captured request or attempt context.</p></div>}
      {tab === "models" && <div className="fields"><label className="field"><span className="field-label">Default model</span><ModelSelect value={draft.model} catalog={catalog} onChange={(model) => { const selected = catalog.find((item) => item.id === model); setDraft((previous) => ({ ...previous, model, effort: selected?.defaultEffort || selected?.efforts[0] || previous.effort })); }} label="Default model" /></label><label className="field"><span className="field-label">Effort</span><EffortSelect value={draft.effort} catalog={catalog} modelId={draft.model} onChange={(value) => set("effort", value)} label="Default effort" /></label>
        {modelError && <p className="error" role="status">{modelError}</p>}{(capabilities?.limitations ?? []).map((item) => <p key={item} className="help">{item}</p>)}
        <p className="help">{catalog.length ? "Models and efforts come from authenticated Codex metadata." : "No authenticated Codex model catalog is available."} One planner and one worker; unsupported restrictions block execution. Cost remains unknown when the provider does not report it.</p>
        {[{ key: "maxTurnSeconds", label: "Maximum seconds per turn", max: 3600 }, { key: "maxOutputTokens", label: "Observed output token stop threshold", max: 100000 }, { key: "maxAttempts", label: "Maximum attempts", max: 10 }, { key: "reviewQueueCap", label: "Open review limit", max: 20 }].map((item) => <label className="field" key={item.key}><span className="field-label">{item.label}</span><input type="number" required min={1} max={item.max} value={draft[item.key]} onChange={(event) => set(item.key, Number(event.target.value))} /></label>)}<p className="help">The token threshold requests a stop after provider usage is observed. It is not a hard generation or spending cap.</p></div>}
      {tab === "permissions" && <div className="fields"><h2 className="section-title">Project permissions</h2><p className="help">Ceilings for every agent in {project.name}. Next-send overrides may restrict these rules. Commands set to Ask require scoped approval for an attempt and its verification commands. This does not prompt for every individual command.</p><PermissionFields describe value={draft} onChange={(values) => setDraft((previous) => ({ ...previous, access: values.access, permissions: values.permissions }))} /><p className="help">This alpha narrows network and integrations to Deny for execution and records that authorization on each attempt. A worktree separates edits; it is not a security sandbox. Provider controls enforce supported operations. No push, merge, deployment or publishing is connected.</p></div>}
      {tab === "files" && <div className="fields"><h2 className="section-title">Repository context</h2><p className="mono live-path">{project.repoPath}</p><p className="help">The runtime captures the goal and revisioned steering before planning and dispatch. Attempt Settings displays the immutable context and worktree identity. Client-side file editing and custom context selection are not connected.</p></div>}
      <footer className="form-foot"><Notice text={notice} /><span className="spacer" />{tab !== "files" && <><button type="button" className="btn" disabled={busy} onClick={() => { setDraft(project.settings); setGoal(project.objective); }}>Cancel</button><button type="submit" className="btn primary" disabled={busy || (tab === "general" ? goal === project.objective || !goal.trim() : !dirty || (tab === "models" && !!modelError))}>{tab === "general" ? "Save goal" : tab === "permissions" ? "Save project permissions" : "Save defaults"}</button></>}</footer>
      </form>}
    </div></div>;
}

function LiveSchedule({ requests, tasks, runs, requestId, navigate, newOneOff, busy, notice, cancel }) {
  const selected = requests.find((item) => item.id === requestId);
  const label = liveScheduleLabel;
  const linkedTasks = selected ? tasks.filter((task) => task.requestId === selected.id) : [];
  const linkedRuns = selected ? runs.filter((run) => run.requestId === selected.id) : [];
  return <div className={`runs schedule ${requestId ? "request-selected" : ""}`}><div className="toolbar"><h1 className="pane-title">Schedule</h1><span className="muted small">Intent for the lead · execution appears in Runs</span><span className="spacer" /><button type="button" className="btn sm" onClick={newOneOff}>New one-off</button></div><Notice text={notice} />
    <div className={`schedule-workspace ${requestId ? "has-selection" : ""}`}><div className="schedule-list">{!requests.length ? <div className="empty-state"><p>No one-off requests yet.</p></div> : <div className="table-scroll"><table className="run-table request-table"><thead><tr><th>Request</th><th>When</th><th>Status</th></tr></thead><tbody>{requests.map((request) => <tr key={request.id}><td><button className="row-link" onClick={() => navigate(`runs/request/${request.id}`)}>{request.text.split("\n")[0]}</button></td><td>{label(request)}</td><td>{request.status.replaceAll("_", " ")}{request.reason && <p className="small muted">{request.reason}</p>}</td></tr>)}</tbody></table></div>}</div>
      {requestId && <aside className="request-detail"><button type="button" className="btn ghost sm" onClick={() => navigate("runs/schedule")}><ArrowLeft size={13} /> Schedule</button>{selected ? <><h2 className="request-heading">One-off request</h2><p className="request-text">{selected.text}</p><dl className="request-facts"><dt>Identity</dt><dd className="mono">{selected.id}</dd><dt>Status</dt><dd>{selected.status}</dd><dt>When</dt><dd>{label(selected)}</dd><dt>Saved time zone</dt><dd>{selected.timeZone || "Not captured"}</dd><dt>Absolute earliest start</dt><dd>{selected.scheduledAt || "Lead policy"}</dd><dt>Captured context</dt><dd>Revision {selected.contextRevision}</dd><dt>Source</dt><dd>{selected.source}</dd></dl>{selected.reason && <p className="help">{selected.reason}</p>}<button type="button" className="btn sm" disabled={busy || !["pending", "waiting"].includes(selected.status) || linkedTasks.length > 0} onClick={() => cancel(selected.id)}>Cancel future dispatch</button><p className="help">Cancellation never interrupts an existing attempt.</p>
        <h3 className="section-title">Linked work</h3>{linkedRuns.length ? linkedRuns.map((run) => <p key={run.id}><a className="text-link mono" href={`#/runs/${run.id}`}>{run.id} · Attempt {run.attempt} · {run.status}</a></p>) : <p className="muted small">No execution attempt exists for this request.</p>}<h3 className="section-title">Requested settings</h3><pre className="live-json">{JSON.stringify(selected.requestedSettings, null, 2)}</pre><h3 className="section-title">Resolved at intake</h3><pre className="live-json">{JSON.stringify(selected.settings, null, 2)}</pre><h3 className="section-title">Context at send</h3><pre className="live-json">{JSON.stringify(selected.context, null, 2)}</pre></> : <p>Request unavailable from this service.</p>}</aside>}
    </div></div>;
}
