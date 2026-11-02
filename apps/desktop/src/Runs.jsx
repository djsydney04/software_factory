import { useEffect, useRef, useState } from "react";
import {
  ArrowLeft, CheckCircle, XCircle, Circle, Clock, Warning, Image, FileText, FilmStrip, MagnifyingGlass, File, ArrowSquareOut,
} from "@phosphor-icons/react";
import {
  NOW, FILTERS, STATUS, ACTIVE, TERMINAL_RETRY, SOURCE_LABEL, clock, dur, modelLabel, effortLabel, modeLabel,
  accessLabel, validateCombo, captureContext,
} from "./data.js";
import { Tabs, Dialog, Status, Notice, ModelSelect, EffortSelect, ModeSelect, ApprovalActions } from "./ui.jsx";
import { SchedulePage } from "./Schedule.jsx";
import { PERMISSION_FIELDS, permissionLabel } from "./permissions.js";

export function RunsPage(props) {
  const { runs, runId, navigate } = props;
  if (props.requestId || (props.ui.view === "schedule" && !runId)) return <SchedulePage {...props} />;
  if (runId) {
    const run = runs.find((r) => r.id === runId);
    if (!run)
      return (
        <div className="empty-state">
          <h1 className="pane-title">Run {runId} isn't available</h1>
          <button type="button" className="btn" onClick={() => navigate("runs")}>Back to Runs</button>
        </div>
      );
    return <RunDetail {...props} run={run} />;
  }
  return <RunsIndex {...props} />;
}

const titleOf = (tasks, run) => tasks.find((t) => t.id === run.taskId)?.title ?? run.taskId;

function RunsIndex({ runs, tasks, ui, setUi, navigate }) {
  useEffect(() => {
    if (!ui.returnFocus) return;
    document.querySelector(`[data-run="${ui.returnFocus}"]`)?.focus();
    setUi((u) => ({ ...u, returnFocus: null }));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const q = ui.query.trim().toLowerCase();
  const filter = FILTERS.find((f) => f.id === ui.filter);
  const matchQ = (r) =>
    !q || [r.id, titleOf(tasks, r), modelLabel(r.settings.model), STATUS[r.status]].some((s) => s.toLowerCase().includes(q));
  const visible = runs.filter((r) => filter.match(r.status) && matchQ(r)).sort((a, b) => b.startedAt - a.startedAt || b.attempt - a.attempt);
  const open = (id) => navigate(`runs/${id}`);

  return (
    <div className="runs">
      <div className="toolbar">
        <h1 className="pane-title">Runs</h1>
        <div className="filters" role="group" aria-label="Filter runs by status">
          {FILTERS.map((f) => (
            <button key={f.id} type="button" className="filter" aria-pressed={ui.filter === f.id} onClick={() => setUi((u) => ({ ...u, filter: f.id }))}>
              {f.label} <span className="count">{runs.filter((r) => f.match(r.status) && matchQ(r)).length}</span>
            </button>
          ))}
        </div>
        <span className="spacer" />
        <label className="search">
          <MagnifyingGlass size={14} aria-hidden="true" />
          <span className="sr-only">Search runs</span>
          <input type="search" placeholder="Search runs" value={ui.query} onChange={(e) => setUi((u) => ({ ...u, query: e.target.value }))} />
        </label>
      </div>
      <div id="runsview-panel" role="tabpanel" aria-labelledby={`runsview-tab-${ui.view}`} className="runs-body">
        {visible.length === 0 ? (
          <div className="empty-state">
            <p>No runs match{q ? ` “${ui.query}”` : ""} in {filter.label}.</p>
            <button type="button" className="btn sm" onClick={() => setUi((u) => ({ ...u, filter: "all", query: "" }))}>Clear filters</button>
          </div>
        ) : ui.view === "list" ? (
          <RunList runs={visible} tasks={tasks} open={open} />
        ) : (
          <RunTimeline runs={visible} tasks={tasks} open={open} />
        )}
      </div>
    </div>
  );
}

const span = (r) => `${clock(r.startedAt)}–${r.endedAt ? clock(r.endedAt) : "now"}`;
const elapsed = (r) => dur(Math.floor(((r.endedAt ?? (r.live ? Date.now() : NOW)) - r.startedAt) / (r.live ? 60_000 : 1)));

function RunList({ runs, tasks, open }) {
  return (
    <div className="table-scroll">
      <table className="run-table">
        <thead>
          <tr>
            <th scope="col">Run</th>
            <th scope="col">Status</th>
            <th scope="col">Model</th>
            <th scope="col">Attempt</th>
            <th scope="col">Time</th>
          </tr>
        </thead>
        <tbody>
          {runs.map((r) => (
            <tr key={r.id}>
              <td className="c-title">
                <button type="button" className="row-link" data-run={r.id} onClick={() => open(r.id)}>
                  {titleOf(tasks, r)}
                </button>
              </td>
              <td><Status status={r.status} /></td>
              <td className="muted">{modelLabel(r.settings.model)}</td>
              <td><span className="muted">Attempt {r.attempt} · </span><span className="mono">{r.id}</span></td>
              <td className="mono muted">{span(r)} <span className="dim">· {elapsed(r)}</span></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

const T0 = 8 * 60 + 30;
const T1 = 12 * 60 + 15;
const pct = (m) => `${((m - T0) / (T1 - T0)) * 100}%`;

function RunTimeline({ runs, tasks, open }) {
  const live = runs.some((run) => run.live);
  const now = live ? Date.now() : NOW;
  const start = live ? Math.min(...runs.map((run) => run.startedAt), now) - 60_000 : T0;
  const end = live ? Math.max(now + 60_000, start + 5 * 60_000) : T1;
  const position = live ? (value) => `${Math.max(0, Math.min(100, ((value - start) / (end - start)) * 100))}%` : pct;
  const ticks = live ? [start, start + (end - start) / 3, start + 2 * (end - start) / 3] : [9, 10, 11, 12].filter((h) => Math.abs(h * 60 - NOW) > 20).map((h) => h * 60);
  return (
    <div className="table-scroll">
      <div className="timeline">
        <div className="tl-axis" aria-hidden="true">
          <span className="tl-label-col" />
          <span className="tl-track">
            {ticks.map((time) => (
              <span key={time} className="tl-hour" style={{ left: position(time) }}>{clock(time)}</span>
            ))}
            <span className="tl-now-label" style={{ left: position(now) }}>Now {clock(now)}</span>
          </span>
        </div>
        <ul className="tl-rows">
          {runs.map((r) => (
            <li key={r.id}>
              <button
                type="button"
                className="tl-row"
                data-run={r.id}
                onClick={() => open(r.id)}
                aria-label={`${titleOf(tasks, r)}, ${STATUS[r.status]}, Attempt ${r.attempt}, ${r.id}, ${span(r)}`}
              >
                <span className="tl-label-col">
                  <span className="tl-title">{titleOf(tasks, r)}</span>
                  <span className="mono dim">A{r.attempt} · {r.id}</span>
                </span>
                <span className="tl-track">
                  {ticks.map((time) => <span key={time} className="tl-grid" style={{ left: position(time) }} />)}
                  <span className="tl-now" style={{ left: position(now) }} />
                  {r.status === "queued" ? (
                    <span className={`tl-queued st-${r.status}`} style={{ left: position(r.startedAt) }} />
                  ) : (
                    <span
                      className={`tl-bar st-${r.status} ${r.endedAt ? "" : "open"}`}
                      style={{ left: position(r.startedAt), width: `calc(${position(r.endedAt ?? now)} - ${position(r.startedAt)})` }}
                    />
                  )}
                </span>
              </button>
            </li>
          ))}
        </ul>
        <p className="help tl-help">{live ? "Observed attempts in local time; open bars end at now." : `Bars end at the sample clock (${clock(NOW)}); nothing after now is drawn.`}</p>
      </div>
    </div>
  );
}

const DETAIL_TABS = ["activity", "files", "checks", "artifacts", "settings"];

function RunDetail({ run, runs, tasks, ui, setUi, navigate, settings, retry, interrupt, cancelQueued, decisions, decide, notice, openReview }) {
  const h1 = useRef(null);
  const [retryOpen, setRetryOpen] = useState(false);
  const [restore, setRestore] = useState(true);
  const siblings = runs.filter((r) => r.taskId === run.taskId).sort((a, b) => a.attempt - b.attempt);
  const active = siblings.find((r) => ACTIVE.includes(r.status));
  const title = titleOf(tasks, run);
  const nextAttempt = Math.max(...siblings.map((r) => r.attempt)) + 1;

  useEffect(() => {
    h1.current?.focus();
  }, [run.id]);

  const back = () => {
    setUi((u) => ({ ...u, returnFocus: run.id }));
    navigate("runs");
  };

  let action = null;
  if (run.status === "queued")
    action = <button type="button" className="btn" onClick={() => cancelQueued(run.id)}>Cancel run</button>;
  else if (run.status === "running" || (run.live && ["claimed", "checking", "needs_approval"].includes(run.status)))
    action = <button type="button" className="btn" onClick={() => interrupt(run.id)}>Interrupt</button>;
  else if (run.status === "interrupting")
    action = <button type="button" className="btn" disabled>Interrupting…</button>;
  else if (TERMINAL_RETRY.includes(run.status))
    action = run.live ? <span className="muted small">Send a new one-off for follow-up work.</span> : active ? (
      <span className="muted small">Retry unavailable · Attempt {active.attempt} is {STATUS[active.status].toLowerCase()}</span>
    ) : (
      <button type="button" className="btn primary" onClick={() => { setRestore(true); setRetryOpen(true); }}>Retry…</button>
    );
  else if (run.status === "succeeded" && run.reviewId)
    action = <button type="button" className="btn" onClick={() => openReview(run.reviewId)}>Open in Reviews</button>;

  const tabItems = [
    { id: "activity", label: "Activity" },
    { id: "files", label: "Files", count: run.changed.length },
    { id: "checks", label: "Checks", count: run.checks.length },
    { id: "artifacts", label: "Artifacts", count: run.artifacts.length },
    { id: "settings", label: "Settings" },
  ];
  const tab = DETAIL_TABS.includes(ui.tab) ? ui.tab : "activity";

  return (
    <div className="run-detail">
      <header className="run-head">
        <div className="run-head-row">
          <button type="button" className="btn ghost sm back" onClick={back}>
            <ArrowLeft size={14} aria-hidden="true" /> Runs
          </button>
          <h1 ref={h1} tabIndex={-1} className="run-title">{title}</h1>
          <span className="mono run-id">{run.id}</span>
          <Status status={run.status} />
          <span className="spacer" />
          {siblings.length > 1 && (
            <label className="history">
              <span className="muted">History</span>
              <select value={run.id} onChange={(e) => navigate(`runs/${e.target.value}`)}>
                {siblings.map((s) => (
                  <option key={s.id} value={s.id}>Attempt {s.attempt} · {s.id} · {STATUS[s.status]}</option>
                ))}
              </select>
            </label>
          )}
          {action}
        </div>
        <p className="run-sub">
          Attempt {run.attempt} · {modelLabel(run.settings.model)} · started {clock(run.startedAt)} ·{" "}
          {run.endedAt ? `ended ${clock(run.endedAt)}` : `updated ${clock(run.updatedAt)}`} · revision{" "}
          <span className="mono">{run.revision}</span>
          {run.retriedFrom && <> · retry of <span className="mono">{run.retriedFrom}</span></>}
        </p>
        <Notice text={notice} />
      </header>
      <Tabs id="runtab" label="Run detail" items={tabItems} value={tab} onChange={(t) => setUi((u) => ({ ...u, tab: t }))} className="underline" />
      <div className="run-panel" id="runtab-panel" role="tabpanel" aria-labelledby={`runtab-tab-${tab}`} tabIndex={0}>
        {tab === "activity" && <ActivityTab run={run} decisions={decisions} decide={decide} runs={runs} />}
        {tab === "files" && <FilesTab run={run} openReview={openReview} />}
        {tab === "checks" && <ChecksTab run={run} runs={runs} />}
        {tab === "artifacts" && <ArtifactsTab run={run} />}
        {tab === "settings" && <SettingsTab run={run} settings={settings} />}
      </div>

      <Dialog id="retry" open={retryOpen} onClose={() => setRetryOpen(false)} title={`Retry as Attempt ${nextAttempt}`} restore={restore}>
        <RetryForm
          run={run}
          title={title}
          settings={settings}
          attempt={nextAttempt}
          onCancel={() => setRetryOpen(false)}
          onConfirm={(cfg) => {
            const id = retry(run, cfg);
            setRestore(false);
            setRetryOpen(false);
            if (id) navigate(`runs/${id}`);
          }}
        />
      </Dialog>
    </div>
  );
}

function ActivityTab({ run, runs, decisions, decide }) {
  return (
    <>
      {run.status === "needs_approval" && run.approval && (
        <div className="callout warn">
          <p><strong>Needs approval:</strong> {run.approval.request}</p>
          <p className="muted small">Scope: {run.approval.scope} · requested {clock(run.approval.requestedAt)} · retry is unavailable while this is open.</p>
          <ApprovalActions runId={run.id} runs={runs} decisions={decisions} decide={decide} showOpen={false} />
        </div>
      )}
      {run.status === "needs_approval" && !run.approval && <p className="callout warn">Waiting for approval evidence. No permission is granted; reconnect to inspect the scoped request.</p>}
      {run.failure && (
        <div className={`callout ${run.status === "failed" ? "danger" : ""}`}>
          <p>{run.failure}</p>
        </div>
      )}
      <ol className="activity">
        {run.activity.map((a, i) => (
          <li key={i}>
            <span className="mono time">{clock(a.at)}</span>
            <div>
              <p>{a.text}</p>
              {a.tools?.length > 0 && (
                <details>
                  <summary>{a.tools.length} tool {a.tools.length === 1 ? "call" : "calls"}</summary>
                  <ul className="tools">
                    {a.tools.map((t) => <li key={t} className="mono">{t}</li>)}
                  </ul>
                </details>
              )}
            </div>
          </li>
        ))}
      </ol>
      <p className="help">Progress summaries and tool evidence. Private model reasoning isn't shown.</p>
    </>
  );
}

function FilesTab({ run, openReview }) {
  return (
    <>
      <section className="panel-section">
        <h2 className="section-title">Changed in this run <span className="muted">· revision <span className="mono">{run.revision}</span></span></h2>
        {run.changed.length ? (
          <ul className="file-list">
            {run.changed.map((f) => (
              <li key={f.path}>
                <File size={14} aria-hidden="true" />
                <span className="mono">{f.path}</span>
                <span className="mono add">+{f.add}</span>
                <span className="mono del">−{f.del}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted">No file changes recorded for this run.</p>
        )}
        {run.reviewId && (
          <button type="button" className="btn sm" onClick={() => openReview(run.reviewId)}>Open full diff in Reviews</button>
        )}
      </section>
      <section className="panel-section">
        <h2 className="section-title">Context inputs <span className="muted">· read, not changed</span></h2>
        <ul className="file-list">
          {run.settings.contextFiles.map((p) => (
            <li key={p}>
              <File size={14} aria-hidden="true" />
              <span className="mono">{p}</span>
              <span className="muted small">context</span>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}

const RESULT = {
  pass: { label: "Pass", Icon: CheckCircle, cls: "ok" },
  fail: { label: "Fail", Icon: XCircle, cls: "bad" },
  running: { label: "Running", Icon: Circle, cls: "info" },
  pending: { label: "Pending", Icon: Clock, cls: "muted" },
  unknown: { label: "Unknown", Icon: Warning, cls: "warn" },
};

function ChecksTab({ run, runs }) {
  if (!run.checks.length) return <p className="muted">No checks have started for this run.</p>;
  const historical = run.live && run.reviewId && run.reviewSourceCurrent !== true;
  return (
    <>
      <p className="scope-line">Selected revision <span className="mono">{run.revision}</span></p>
      {historical && <p className="help">Historical evidence for the pinned revision. {run.reviewSourceReason} Local review decisions are blocked.</p>}
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr><th scope="col">Check</th><th scope="col">Result</th><th scope="col">Observed</th><th scope="col">Tested revision</th></tr>
          </thead>
          <tbody>
            {run.checks.map((c) => {
              const stale = historical || c.revision !== run.revision;
              const R = RESULT[c.result];
              const from = runs.find((r) => r.revision === c.revision && r.id !== run.id);
              return (
                <tr key={c.name + c.revision} className={stale ? "stale" : ""}>
                  <td>{c.name}{run.live && <details className="small"><summary>Command and output</summary><p className="mono">{c.program} {(c.args ?? []).join(" ")}</p><p>Exit: {c.exitCode ?? "not observed"}{c.timedOut ? " · timed out" : ""}</p><pre className="live-json">{c.output || "No output captured."}</pre></details>}</td>
                  <td>
                    <span className={`result ${stale ? "muted" : R.cls}`}><R.Icon size={14} aria-hidden="true" /> {R.label}</span>
                    {stale && <span className="stale-tag"> · {historical ? "historical" : "stale"}</span>}
                  </td>
                  <td className="mono muted">{c.at != null ? clock(c.at) : "—"}</td>
                  <td>
                    <span className="mono">{c.revision}</span>
                    {stale && (
                      <span className="muted small"> · {historical ? "does not certify the current worktree" : `${from ? `from ${from.id}` : "older revision"}; doesn't certify ${run.revision}`}</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </>
  );
}

const ART_ICON = { screenshot: Image, report: FileText, recording: FilmStrip };
const ART_STATE = { current: "Current", stale: "Stale", missing: "Missing", expired: "Expired" };

function ArtifactsTab({ run }) {
  if (!run.artifacts.length) return <p className="muted">No artifacts for this run.</p>;
  return (
    <>
      <ul className="artifact-list">
        {run.artifacts.map((a) => {
          const Icon = ART_ICON[a.kind] ?? FileText;
          const state = a.state === "current" && a.revision !== run.revision ? "stale" : a.state;
          const linkable = a.href && (state === "current" || state === "stale");
          return (
            <li key={a.label}>
              <Icon size={16} aria-hidden="true" />
              <span className="art-label">{a.label}</span>
              <span className={`art-state ${state}`}>{ART_STATE[state]}</span>
              <span className="mono muted">{a.revision}</span>
              {a.note && <span className="muted small">{a.note}</span>}
              <span className="spacer" />
              {linkable && (
                <a className="text-link small" href={a.href} target="_blank" rel="noreferrer">
                  Open sample <ArrowSquareOut size={12} aria-hidden="true" />
                </a>
              )}
            </li>
          );
        })}
      </ul>
      <p className="help">Sample artifacts bound to this run and revision. Stale items came from an older revision.</p>
    </>
  );
}

function SettingsTab({ run, settings }) {
  if (run.live) return <><p className="scope-line"><strong>Attempt snapshots</strong> · read only · context revision {run.contextRevision}</p>
    <dl className="request-facts"><dt>Provider thread</dt><dd className="mono">{run.threadId ?? "Not observed"}</dd><dt>Provider turn</dt><dd className="mono">{run.turnId ?? "Not observed"}</dd><dt>Worktree</dt><dd className="mono">{run.worktreePath ?? "Not created"}</dd><dt>Base revision</dt><dd className="mono">{run.baseSha ?? "Not captured"}</dd><dt>Head revision</dt><dd className="mono">{run.headSha ?? "Not captured"}</dd></dl>
    <h2 className="section-title">Requested settings</h2><pre className="live-json">{JSON.stringify(run.requestedSettings, null, 2)}</pre>
    <h2 className="section-title">Dispatch ceilings</h2><p className="help">Resolved when the attempt was claimed. Execution may be narrower.</p><pre className="live-json">{JSON.stringify(run.effectiveSettings, null, 2)}</pre>
    <h2 className="section-title">Execution authorization</h2>{run.authorization ? <><p className="help">Settings used for dispatch, with the scoped approval identities and enforced limitations captured before execution.</p><pre className="live-json">{JSON.stringify(run.authorization, null, 2)}</pre></> : <p className="muted">No execution authorization has been recorded for this attempt.</p>}
    <h2 className="section-title">Captured context</h2><pre className="live-json">{JSON.stringify(run.context, null, 2)}</pre></>;
  const s = run.settings;
  const src = (k) => (s.source[k] === "copied" && run.retriedFrom ? `Copied from ${run.retriedFrom}` : SOURCE_LABEL[s.source[k]]);
  const rows = [
    ["Provider / model", modelLabel(s.model), src("model")],
    ["Effort", effortLabel(s.effort), src("effort")],
    ["Execution mode", modeLabel(s.mode), src("mode")],
    ["Agent edits", accessLabel(s.access) ?? "Not captured", "Captured"],
    ...PERMISSION_FIELDS.map(({ id, label }) => [label, permissionLabel(s.permissions?.[id]), s.permissions ? "Captured" : "Not captured"]),
    ["Context revision", <span className="mono">{s.contextRevision}</span>, "Captured"],
    ["Context files", <span className="mono">{s.contextFiles.join(", ")}</span>, "Captured"],
  ];
  if (s.roots) rows.push(["Context roots", <span className="mono">{s.roots.join(", ")}</span>, "Project default at submit"]);
  return (
    <>
      <p className="scope-line">
        <strong>Selected run · Effective settings</strong> · <span className="mono">{run.id}</span> · captured {clock(s.capturedAt)} · read only
      </p>
      <div className="table-scroll">
        <table className="data-table kv-table">
          <thead><tr><th scope="col">Setting</th><th scope="col">Value</th><th scope="col">Source</th></tr></thead>
          <tbody>
            {rows.map(([k, v, from]) => (
              <tr key={k}><th scope="row">{k}</th><td>{v}</td><td className="muted">{from}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="help">
        Project defaults are now {modelLabel(settings.model)} · {effortLabel(settings.effort)} · {modeLabel(settings.mode)}. They apply to future runs; this snapshot never changes.{" "}
        <a className="text-link" href="#/settings">Project settings</a>
      </p>
    </>
  );
}

function RetryForm({ run, title, settings, attempt, onCancel, onConfirm }) {
  const base = (b) =>
    b === "original"
      ? { model: run.settings.model, effort: run.settings.effort, mode: run.settings.mode }
      : { model: settings.model, effort: settings.effort, mode: settings.mode };
  const [basis, setBasis] = useState("original");
  const [vals, setVals] = useState(() => base("original"));
  const [edited, setEdited] = useState({});
  const busy = useRef(false);
  const err = validateCombo(vals.model, vals.effort);
  const source = (k) => (edited[k] ? "edit" : basis === "original" ? "copied" : "default");
  const choose = (b) => {
    setBasis(b);
    setVals(base(b));
    setEdited({});
  };
  const setV = (k, v) => {
    if (!v) return;
    setVals((x) => ({ ...x, [k]: v }));
    setEdited((e) => ({ ...e, [k]: true }));
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (busy.current || err) return;
        busy.current = true;
        onConfirm({
          ...vals,
          source: { model: source("model"), effort: source("effort"), mode: source("mode") },
          ...(basis === "defaults" ? { access: settings.access, permissions: structuredClone(settings.permissions), ...captureContext(settings) } : {}),
        });
      }}
    >
      <p>
        Creates a new run for “{title}”. <span className="mono">{run.id}</span> and its evidence stay unchanged.
      </p>
      <fieldset className="plain">
        <legend className="field-label">Start from</legend>
        <label className="check">
          <input type="radio" name="basis" checked={basis === "original"} onChange={() => choose("original")} />
          <span><span className="mono">{run.id}</span> settings ({modelLabel(run.settings.model)} · {effortLabel(run.settings.effort)})</span>
        </label>
        <label className="check">
          <input type="radio" name="basis" checked={basis === "defaults"} onChange={() => choose("defaults")} />
          <span>Current project defaults ({modelLabel(settings.model)} · {effortLabel(settings.effort)})</span>
        </label>
      </fieldset>
      <div className="next-run-grid">
        <label><span>Model</span><ModelSelect value={vals.model} onChange={(v) => setV("model", v)} aria-invalid={!!err} aria-describedby={err ? "retry-combo-err" : undefined} /></label>
        <label><span>Effort</span><EffortSelect value={vals.effort} onChange={(v) => setV("effort", v)} modelId={vals.model} aria-invalid={!!err} aria-describedby={err ? "retry-combo-err" : undefined} /></label>
        <label><span>Mode</span><ModeSelect value={vals.mode} onChange={(v) => setV("mode", v)} /></label>
      </div>
      {err ? (
        <p className="error" id="retry-combo-err" role="alert">{err}</p>
      ) : (
        <p className="help">
          {Object.keys(edited).length ? `Edited for this attempt only: ${Object.keys(edited).join(", ")}.` : "No edits. Values are used exactly as shown."}
        </p>
      )}
      <div className="dialog-foot">
        <button type="button" className="btn" onClick={onCancel}>Cancel</button>
        <button type="submit" className="btn primary" disabled={!!err}>Create Attempt {attempt}</button>
      </div>
    </form>
  );
}
