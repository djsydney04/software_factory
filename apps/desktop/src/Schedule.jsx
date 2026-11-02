import { useEffect, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, MagnifyingGlass, Plus } from "@phosphor-icons/react";
import { accessLabel, effortLabel, modelLabel, modeLabel } from "./data.js";
import { Dialog, Notice } from "./ui.jsx";
import { PermissionFacts } from "./Permissions.jsx";
import { POLICIES, cancelRequest, formatRequestTime, localInputValue, localZone, policyLabel, requestState, rescheduleRequest, validateTiming } from "./scheduling.js";

const FILTERS = [{ id: "all", label: "All" }, { id: "waiting", label: "Waiting" }, { id: "due", label: "Due" }, { id: "cancelled", label: "Cancelled" }];

export function SchedulePage({ requests, setRequests, requestId, ui, setUi, navigate, newOneOff, storeError }) {
  const [now, setNow] = useState(Date.now);
  const [notice, setNotice] = useState("");
  const [rescheduling, setRescheduling] = useState(false);
  const title = useRef(null);
  const scheduleTitle = useRef(null);
  const filter = ui.scheduleFilter ?? "all";
  const query = ui.scheduleQuery ?? "";
  const selected = requests.find((r) => r.id === requestId);
  const matchQuery = (r) => !query.trim() || [r.text, r.id, modelLabel(r.settings.model), policyLabel(r), requestState(r, now).label].some((s) => s.toLowerCase().includes(query.trim().toLowerCase()));
  const matchFilter = (r, f) => f === "all" || (f === "waiting" ? ["waiting", "scheduled"].includes(requestState(r, now).id) : requestState(r, now).id === f);
  const visible = requests.filter((r) => matchQuery(r) && matchFilter(r, filter)).sort((a, b) => b.createdAt - a.createdAt);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 60_000);
    const update = () => setNow(Date.now());
    window.addEventListener("focus", update);
    return () => { window.clearInterval(timer); window.removeEventListener("focus", update); };
  }, []);

  useEffect(() => { if (requestId) title.current?.focus(); }, [requestId]);

  useEffect(() => {
    if (!requestId && ui.requestReturnFocus) {
      const row = document.querySelector(`[data-request="${CSS.escape(ui.requestReturnFocus)}"]`);
      (row ?? scheduleTitle.current)?.focus();
      setUi((u) => ({ ...u, requestReturnFocus: null }));
    }
  }, [requestId]); // eslint-disable-line react-hooks/exhaustive-deps

  const closeDetail = () => {
    setUi((u) => ({ ...u, view: "schedule", requestReturnFocus: requestId }));
    navigate("runs/schedule");
  };
  const update = (request, text) => {
    setRequests((rs) => rs.map((r) => r.id === request.id ? request : r));
    setNow(Date.now());
    setNotice(text);
  };

  return (
    <div className={`runs schedule ${requestId ? "request-selected" : ""}`}>
      <div className="toolbar">
        <h1 ref={scheduleTitle} tabIndex={-1} className="pane-title">Schedule</h1>
        <div className="filters" role="group" aria-label="Filter one-off requests">
          {FILTERS.map((f) => <button key={f.id} type="button" className="filter" aria-pressed={filter === f.id} onClick={() => setUi((u) => ({ ...u, scheduleFilter: f.id }))}>{f.label} <span className="count">{requests.filter((r) => matchQuery(r) && matchFilter(r, f.id)).length}</span></button>)}
        </div>
        <span className="spacer" />
        <label className="search"><MagnifyingGlass size={14} aria-hidden="true" /><span className="sr-only">Search requests</span><input type="search" placeholder="Search requests" value={query} onChange={(e) => setUi((u) => ({ ...u, scheduleQuery: e.target.value }))} /></label>
        <button type="button" className="btn sm" onClick={newOneOff}><Plus size={14} aria-hidden="true" /> New one-off</button>
      </div>
      <div id="runsview-panel" role="tabpanel" aria-labelledby="runsview-tab-schedule" className={`schedule-workspace ${requestId ? "has-selection" : ""}`}>
        <div className="schedule-list">
          <div className="schedule-intro"><p className="muted small">One-off requests for the Factory lead. Execution attempts appear in List and Timeline.</p></div>
          {storeError && <p className="error small schedule-feedback" role="alert">Local storage is unavailable. Requests are kept in memory and may be lost on reload.</p>}
          <div className="schedule-feedback"><Notice text={notice} /></div>
          {!requests.length ? <div className="empty-state"><p>No one-off requests yet.</p><button type="button" className="btn sm" onClick={newOneOff}>New one-off</button></div>
            : !visible.length ? <div className="empty-state"><p>No requests match these filters.</p><button type="button" className="btn sm" onClick={() => setUi((u) => ({ ...u, scheduleFilter: "all", scheduleQuery: "" }))}>Clear filters</button></div>
            : <div className="table-scroll"><table className="run-table request-table"><thead><tr><th scope="col">Request</th><th scope="col">When</th><th scope="col">Status</th><th scope="col" className="schedule-model">Model</th><th scope="col" className="schedule-created">Created</th></tr></thead><tbody>
              {visible.map((request) => <tr key={request.id} aria-selected={selected?.id === request.id}>
                <td className="request-title"><button type="button" className="row-link" data-request={request.id} onClick={() => navigate(`runs/request/${request.id}`)}>{request.text.split("\n")[0]}</button><span className="mono muted request-id">{request.id}</span></td>
                <td data-label="When">{policyLabel(request)}</td>
                <td data-label="Status"><RequestStatus request={request} now={now} /></td>
                <td className="muted schedule-model" data-label="Model">{modelLabel(request.settings.model)}</td>
                <td className="muted schedule-created" data-label="Created">{formatRequestTime(request.createdAt, request.timeZone)}</td>
              </tr>)}
            </tbody></table></div>}
          <p className="schedule-foot small muted">Design preview: requests are {storeError ? "kept in memory only" : "saved locally"}; nothing starts. The lead will orchestrate work when connected.</p>
        </div>
        {requestId && (selected ? <aside className="request-detail" aria-label="Selected one-off request">
          <header className="request-head"><button type="button" className="btn ghost sm" onClick={closeDetail}><ArrowLeft size={14} aria-hidden="true" /> Schedule</button><span className="mono muted">{selected.id}</span></header>
          <h2 ref={title} tabIndex={-1} className="request-heading">One-off request</h2>
          {storeError && <p className="error small" role="alert">Local storage is unavailable. Changes may be lost on reload.</p>}
          <Notice text={notice} />
          <p className="request-text">{selected.text}</p>
          <dl className="request-facts"><dt>Status</dt><dd><RequestStatus request={selected} now={now} /></dd><dt>Owner</dt><dd>Factory lead</dd>{selected.origin?.type === "scratchpad" && <><dt>Source</dt><dd><a className="text-link" href={`#/scratchpad/${encodeURIComponent(selected.origin.noteId)}`}>Scratchpad note</a> · <span className="mono">{selected.origin.noteId}</span> · revision {selected.origin.revision}</dd></>}<dt>When</dt><dd>{policyLabel(selected)}{selected.policy === "at" && <span className="muted small"> · {selected.timeZone}</span>}</dd><dt>Policy</dt><dd className="muted">{selected.policy === "at" ? "Earliest start, subject to current work and dependencies." : selected.policy === "next" ? "After current work and dependencies; no interruption." : "The lead decides where this fits."}</dd></dl>
          <div className="btn-row request-actions"><button type="button" className="btn sm" disabled={selected.status === "cancelled"} onClick={() => setRescheduling(true)}>Reschedule…</button><button type="button" className="btn sm" disabled={selected.status === "cancelled"} onClick={() => update(cancelRequest(selected), "Request cancelled. No execution run was changed.")}>Cancel request</button></div>
          <section className="request-snapshot"><h3 className="section-title">Requested settings</h3><p className="muted small">{selected.origin?.type === "scratchpad" ? "Captured at preview triage" : "Captured at send"} · read only<br />{formatRequestTime(selected.settings.capturedAt ?? selected.createdAt, selected.timeZone, true)}</p><dl className="request-facts"><dt>Model</dt><dd>{modelLabel(selected.settings.model)}<Scope source={selected.settings.source?.model} /></dd><dt>Effort</dt><dd>{effortLabel(selected.settings.effort)}<Scope source={selected.settings.source?.effort} /></dd><dt>Mode</dt><dd>{modeLabel(selected.settings.mode)}<Scope source={selected.settings.source?.mode} /></dd><dt>Agent edits</dt><dd>{accessLabel(selected.settings.access) ?? "Not captured"}<Scope source={selected.settings.source?.permissions} /></dd><PermissionFacts permissions={selected.settings.permissions} /><dt>Budget</dt><dd>{selected.settings.budget != null ? `$${selected.settings.budget} project limit` : "Not captured"}</dd><dt>Concurrency</dt><dd>{selected.settings.concurrency ?? "Not captured"}</dd></dl><h3 className="section-title">Context at send</h3><p className="muted small">Inputs and permissions remain as captured when project defaults change.</p><ul className="request-context">{selected.context.contextFiles.length ? selected.context.contextFiles.map((path) => <li className="mono" key={path}>{path}{selected.context.attached?.includes(path) && <span className="muted small"> · attached</span>}</li>) : <li className="muted">No context files captured.</li>}</ul><details><summary>Context configuration</summary><p className="small">Roots: {(selected.context.roots ?? []).join(", ") || "None"}</p><p className="small">AGENTS.md: {selected.context.alwaysAgents ? "Included" : "Project rule"}</p><p className="small">Excluded: {(selected.context.excludedFiles ?? []).join(", ") || "None"}</p><pre className="small">Ignore patterns: {selected.context.ignore || "None"}</pre></details></section>
          <a href="#/conversation" className="text-link small">Open conversation <ArrowRight size={12} aria-hidden="true" /></a>
          <p className="small muted request-preview-note">Design preview · request {storeError ? "kept in memory only" : "saved locally"}, nothing starts.</p>
        </aside> : <aside className="request-detail"><p>Request {requestId} is not saved in this preview.</p><button type="button" className="btn sm" onClick={closeDetail}>Back to Schedule</button></aside>)}
      </div>
      {selected && <Dialog id="reschedule-request" open={rescheduling} onClose={() => setRescheduling(false)} title="Reschedule request"><RescheduleForm request={selected} onCancel={() => setRescheduling(false)} onSave={(request) => { update(request, "Request timing updated. Captured settings and context are unchanged."); setRescheduling(false); }} /></Dialog>}
    </div>
  );
}

function Scope({ source }) {
  return <span className="muted small request-source">{source === "override" ? "Next-send override" : "Project default at send"}</span>;
}

function RequestStatus({ request, now }) {
  const state = requestState(request, now);
  return <span className={`request-status request-${state.id}`}><span className="dot" aria-hidden="true" />{state.label}</span>;
}

function RescheduleForm({ request, onCancel, onSave }) {
  const [policy, setPolicy] = useState(request.policy);
  const [localTime, setLocalTime] = useState(request.scheduledAt ? localInputValue(request.scheduledAt) : "");
  const [error, setError] = useState("");
  const busy = useRef(false);
  const timing = validateTiming(policy, localTime);
  return <form noValidate onSubmit={(e) => {
    e.preventDefault();
    if (busy.current) return;
    const result = rescheduleRequest(request, policy, localTime);
    if (result.error) { setError(result.error); return; }
    busy.current = true;
    onSave(result.request);
  }}>
    <p>Change when the lead should fit this request into ongoing work. Captured settings and context stay unchanged.</p>
    <p className="small muted">Current: {policyLabel(request)}{request.policy === "at" ? ` · ${request.timeZone}` : ""}</p>
    <label className="schedule-field"><span>When</span><select value={policy} onChange={(e) => { setPolicy(e.target.value); setError(""); }} aria-label="New scheduling policy">{POLICIES.map((p) => <option key={p.id} value={p.id}>{p.label}</option>)}</select></label>
    {policy === "at" && <label className="schedule-field"><span>New local date and time · {localZone()}</span><input type="datetime-local" value={localTime} onChange={(e) => { setLocalTime(e.target.value); setError(""); }} aria-label="New scheduled local date and time" aria-invalid={!!error} aria-describedby="reschedule-time-help" /></label>}
    <p id="reschedule-time-help" className="muted small">{policy === "at" ? timing.error ? `Enter a future time in the browser zone ${localZone()}.` : `Resolves to ${formatRequestTime(timing.scheduledAt, timing.timeZone, true)} · ${timing.timeZone}.` : policy === "next" ? "The lead waits for current work and dependencies. Existing runs are not interrupted." : "The Factory lead decides where this fits."}</p>
    {error && <p className="error" role="alert">{error}</p>}
    <div className="dialog-foot"><button type="button" className="btn" onClick={onCancel}>Cancel</button><button type="submit" className="btn primary">Save timing</button></div>
  </form>;
}
