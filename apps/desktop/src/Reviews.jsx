import { useEffect, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, CaretLeft, CaretRight, CheckCircle, ImageBroken, MagnifyingGlass, WarningCircle } from "@phosphor-icons/react";
import { REVIEWS, MERGED, clock, model } from "./data.js";
import { Tabs, Dialog, Notice } from "./ui.jsx";
import { reviewEvidence, recordReviewDecision } from "./review-state.js";
import "./reviews.css";

const LABELS = { ready: "Ready", blocked: "Source needs attention", changes: "Changes requested", merged: "Merged locally", accepted: "Accepted locally" };
const RESULT_LABELS = { pass: "Passed", fail: "Failed", running: "Running", pending: "Pending", unknown: "Unknown" };
const sourceOf = (review) => ({ runId: review.runId, attempt: review.attempt, revision: review.revision });
const safeArtifactHref = (value) => typeof value === "string" && /^(https?:\/\/|\/(?!\/)|\.{1,2}\/)/i.test(value) ? value : null;

export function ReviewsPage({ reviewId, navigate, runs, tasks, state, setState, ui, setUi, openRun, storeError, reviews = REVIEWS, mergedHistory = MERGED, onDecision = null, busy = false }) {
  const [dialog, setDialog] = useState(null);
  const [notice, setNotice] = useState("");
  const [decisionError, setDecisionError] = useState("");
  const [deciding, setDeciding] = useState(false);
  const heading = useRef(null);
  const feedback = useRef(null);
  const rowRefs = useRef({});
  const review = reviews.find((item) => item.id === reviewId);
  const live = Boolean(onDecision);
  const filter = ui.filter ?? "all";
  const query = ui.query ?? "";
  const statuses = state.status ?? {};
  const evidenceFor = (item) => reviewEvidence(item, runs, tasks, statuses[item.id] ?? item.group);
  const rows = reviews.filter((item) => {
    const status = statuses[item.id] ?? item.group;
    const matchesFilter = filter === "all" ? !["merged", "accepted"].includes(status) : status === filter;
    const evidence = evidenceFor(item);
    return matchesFilter && `${evidence.title} ${item.summary} ${item.number} ${item.id} ${evidence.provider}`.toLowerCase().includes(query.trim().toLowerCase());
  });
  const history = filter === "merged" ? mergedHistory.filter((item) => item.toLowerCase().includes(query.trim().toLowerCase())) : [];

  useEffect(() => {
    setDialog(null);
    setDecisionError("");
    setNotice("");
    if (reviewId) heading.current?.focus();
    else (rowRefs.current[ui.lastRow] ?? heading.current)?.focus();
  }, [reviewId]);
  useEffect(() => {
    if (dialog?.type !== "changes") return;
    const frame = window.requestAnimationFrame(() => feedback.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, [dialog?.type, dialog?.reviewId]);

  const openReview = (id) => {
    setUi((previous) => ({ ...previous, lastRow: id }));
    navigate(`reviews/${id}`);
  };
  const updateView = (patch) => setUi((previous) => ({ ...previous, views: { ...previous.views,
    [reviewId]: { ...previous.views?.[reviewId], ...patch } } }));
  const openDecision = (type) => {
    setDecisionError("");
    const recorded = state.notes?.[review.id] ?? review.requested;
    if (type === "changes" && recorded && !state.drafts?.[review.id]) setState((previous) => ({ ...previous, drafts: { ...previous.drafts, [review.id]: recorded } }));
    setDialog({ type, reviewId: review.id, expectedSource: sourceOf(review) });
  };
  const confirmDecision = async (text) => {
    if (!dialog || deciding || busy) return;
    if (live) {
      const current = reviews.find((item) => item.id === dialog.reviewId);
      if (!current || Object.keys(dialog.expectedSource).some((key) => current[key] !== dialog.expectedSource[key])) {
        setDecisionError("The reviewed source changed. Reopen this decision for its current source."); return;
      }
      const evidence = evidenceFor(current);
      if (!evidence.sourceValid || (dialog.type === "merge" && !evidence.canMerge)) {
        setDecisionError(evidence.blockedReason); return;
      }
      setDeciding(true);
      try {
        await onDecision({ ...dialog, text });
        if (dialog.type === "changes") setState((previous) => ({ ...previous, drafts: { ...previous.drafts, [dialog.reviewId]: "" } }));
        setDialog(null); setDecisionError("");
        setNotice(dialog.type === "merge" ? "Accepted locally on this exact revision. Nothing was merged or published." : "Local change request recorded on this revision.");
        window.requestAnimationFrame(() => heading.current?.focus());
      } catch (error) { setDecisionError(error.message || String(error)); }
      finally { setDeciding(false); }
      return;
    }
    const result = recordReviewDecision(state, { ...dialog, text }, reviews, runs, tasks);
    if (result.error) { setDecisionError(result.error); return; }
    setState(() => result.state);
    setDialog(null);
    setDecisionError("");
    setNotice(dialog.type === "merge" ? "Merge simulated locally. No GitHub call was made."
      : "Change request recorded locally. No comment was posted or agent dispatched.");
    window.requestAnimationFrame(() => heading.current?.focus());
  };

  if (!reviewId) return (
    <section className="rv-inbox" aria-labelledby="rv-inbox-title">
      <header className="rv-inbox-head">
        <div><h1 id="rv-inbox-title" ref={heading} tabIndex={-1}>Reviews</h1>
          <p className="muted small">Inspect the work, then decide what happens next.</p></div>
        <span className="rv-inbox-count small muted">{reviews.filter((item) => !["merged", "accepted"].includes(statuses[item.id] ?? item.group)).length} active</span>
      </header>
      <div className="rv-inbox-tools">
        <label className="rv-search"><MagnifyingGlass size={16} aria-hidden="true" /><span className="sr-only">Search reviews</span>
          <input type="search" placeholder="Search reviews" value={query} onChange={(event) => setUi((previous) => ({ ...previous, query: event.target.value }))} />
        </label>
        <label className="rv-filter"><span className="sr-only">Review status</span>
          <select value={filter} onChange={(event) => setUi((previous) => ({ ...previous, filter: event.target.value }))}>
            <option value="all">All active</option><option value="ready">Ready</option>{live && <option value="blocked">Source needs attention</option>}<option value="changes">Changes requested</option><option value={live ? "accepted" : "merged"}>{live ? "Accepted locally" : "Merged"}</option>
          </select>
        </label>
      </div>
      {storeError && <p className="rv-storage error" role="alert">Local storage is unavailable. Changes stay in memory and may be lost on reload.</p>}
      <div className="rv-inbox-scroll">
        {rows.length > 0 && <ul className="rv-list" aria-label="Reviews">{rows.map((item) => {
          const evidence = evidenceFor(item);
          const status = statuses[item.id] ?? item.group;
          return <li key={item.id}>
            <a href={`#/reviews/${item.id}`} className="rv-row" ref={(element) => { rowRefs.current[item.id] = element; }}
              onClick={(event) => { if (event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) { event.preventDefault(); openReview(item.id); } }}>
              <div className="rv-row-copy"><span className="rv-row-title"><span className="mono muted">#{item.number}</span> {evidence.title}</span>
                <span className="rv-row-summary">{item.summary}</span>
                <span className="rv-row-meta"><span>{evidence.provider}</span><span>{evidence.passed}/{evidence.total} checks passed</span>
                  {evidence.limitations.length > 0 && <span className="warn-text">{evidence.limitations.length} {evidence.limitations.length === 1 ? "limitation" : "limitations"}</span>}
                  {!evidence.sourceValid && <span className="bad">Source unavailable</span>}
                </span>
              </div>
              <span className={`rv-status rv-status-${status}`}><span aria-hidden="true" />{LABELS[status] ?? "Unknown"}</span>
              <ArrowRight className="rv-row-arrow" size={16} aria-hidden="true" />
            </a>
          </li>;
        })}</ul>}
        {history.length > 0 && <section className="rv-history" aria-labelledby="rv-history-title"><h2 id="rv-history-title">Earlier merges</h2>
          <p className="muted small">History only. Source reviews are not available in this preview.</p>
          <ul>{history.map((item) => <li key={item}>{item}</li>)}</ul>
        </section>}
        {!rows.length && !history.length && <div className="rv-empty"><h2>{query ? "No matching reviews" : "No reviews in this view"}</h2>
          <p className="muted">Try another status or clear your search.</p><button type="button" className="btn" onClick={() => setUi((previous) => ({ ...previous, filter: "all", query: "" }))}>Show all active</button></div>}
      </div>
    </section>
  );

  if (!review) return <section className="rv-not-found"><h1 ref={heading} tabIndex={-1}>Review unavailable</h1>
    <p className="muted">Review <span className="mono">{reviewId}</span> is not available{live ? " from this service" : " in this preview"}.</p>
    <button className="btn" type="button" onClick={() => navigate("reviews")}><ArrowLeft size={14} aria-hidden="true" /> Back to Reviews</button></section>;

  const status = statuses[review.id] ?? review.group;
  const evidence = evidenceFor(review);
  const view = ui.views?.[review.id] ?? {};
  const tab = ["preview", "changes", "evidence"].includes(view.tab) ? view.tab : review.preview ? "preview" : "changes";
  const zoom = view.zoom === "actual" ? "actual" : "fit";
  const files = Array.isArray(review.files) ? review.files : [];
  const fileIndex = Number.isInteger(view.file) && view.file >= 0 && view.file < files.length ? view.file : 0;
  const file = files[fileIndex];
  const position = reviews.findIndex((item) => item.id === review.id);
  const requested = state.notes?.[review.id] ?? review.requested;
  const latestDecision = (state.decisions ?? []).filter((item) => item.reviewId === review.id).at(-1);
  const dialogReview = dialog ? reviews.find((item) => item.id === dialog.reviewId) : null;
  const draft = dialog ? state.drafts?.[dialog.reviewId] ?? "" : "";
  const tabId = `rv-${review.id}`;

  return <section className="rv-desk" aria-labelledby="rv-review-title">
    <nav className="rv-desk-nav" aria-label="Review navigation">
      <button type="button" className="link-btn rv-back" onClick={() => navigate("reviews")}><ArrowLeft size={14} aria-hidden="true" /> Reviews</button>
      <div className="rv-stepper"><button type="button" className="icon-btn" aria-label="Previous review" disabled={position === 0} onClick={() => openReview(reviews[position - 1].id)}><CaretLeft size={16} /></button>
        <span className="small muted" aria-label={`Review ${position + 1} of ${reviews.length}`}>{position + 1} of {reviews.length}</span>
        <button type="button" className="icon-btn" aria-label="Next review" disabled={position === reviews.length - 1} onClick={() => openReview(reviews[position + 1].id)}><CaretRight size={16} /></button></div>
    </nav>
    <header className="rv-desk-head">
      <div className="rv-title-line"><h1 id="rv-review-title" ref={heading} tabIndex={-1}>{evidence.title} <span className="mono muted">#{review.number}</span></h1>
        <span className={`rv-status rv-status-${status}`}><span aria-hidden="true" />{LABELS[status] ?? "Unknown"}</span></div>
      <p className="rv-scope small muted"><span className={`rv-mobile-status rv-status rv-status-${status}`}><span aria-hidden="true" />{LABELS[status] ?? "Unknown"}</span>{evidence.run ? <button type="button" className="link-btn" onClick={() => openRun(review.runId)}>Run <span className="mono">{review.runId}</span></button> : <span>Run unavailable</span>}
        <span>Attempt {review.attempt ?? "unknown"}</span><span className="mono">{review.revision ?? "Revision unavailable"}</span><span>{evidence.provider}</span></p>
      <p className="rv-summary">{review.summary}</p>
    </header>
    <div className="rv-modes"><Tabs id={tabId} label="Review content" value={tab} onChange={(value) => updateView({ tab: value })} className="rv-tabs"
      items={[{ id: "preview", label: "Preview" }, { id: "changes", label: "Changes", count: files.length }, { id: "evidence", label: "Evidence" }]} />
      {tab === "preview" && review.preview && <div className="rv-zoom" role="group" aria-label="Preview zoom">
        <button className="rv-zoom-btn" type="button" aria-pressed={zoom === "fit"} onClick={() => updateView({ zoom: "fit" })}>Fit width</button>
        <button className="rv-zoom-btn" type="button" aria-pressed={zoom === "actual"} onClick={() => updateView({ zoom: "actual" })}>100%</button></div>}
    </div>
    {(status === "changes" && requested || !evidence.sourceValid || storeError) && <div className="rv-alerts">
      {status === "changes" && requested && <p className="rv-callout warn-text"><WarningCircle size={15} aria-hidden="true" /><span>Changes requested: {requested} <span className="muted">Waiting for a new revision.</span></span></p>}
      {!evidence.sourceValid && <p className="rv-callout bad" role="alert"><WarningCircle size={15} aria-hidden="true" /><span>{evidence.blockedReason}</span></p>}
      {storeError && <p className="error" role="alert">Local storage is unavailable. Changes stay in memory and may be lost on reload.</p>}
    </div>}
    <div className={`rv-stage rv-stage-${tab}`} id={`${tabId}-panel`} role="tabpanel" aria-labelledby={`${tabId}-tab-${tab}`}>
      {tab === "preview" && <PreviewPane key={review.id} review={review} zoom={zoom} onChanges={() => updateView({ tab: "changes" })} />}
      {tab === "changes" && <DiffPane review={review} file={file} fileIndex={fileIndex} onFile={(value) => updateView({ file: value })} />}
      {tab === "evidence" && <EvidencePane review={review} evidence={evidence} requested={requested} decisions={(state.decisions ?? []).filter((item) => item.reviewId === review.id)} />}
    </div>
    <footer className="rv-decision-bar">
      <div className="rv-readiness"><button type="button" className="rv-evidence-link" onClick={() => updateView({ tab: "evidence" })}>
        {evidence.canMerge ? <CheckCircle size={15} className="ok" aria-hidden="true" /> : <WarningCircle size={15} className={["merged", "accepted"].includes(status) ? "muted" : "warn-text"} aria-hidden="true" />}
        <span>{evidence.passed}/{evidence.total} checks passed{evidence.limitations.length > 0 ? ` · ${evidence.limitations.length} ${evidence.limitations.length === 1 ? "limitation" : "limitations"}` : ""}<span className="rv-evidence-word"> · Evidence</span></span></button>
        {!["merged", "accepted"].includes(status) && !evidence.canMerge && <p className="rv-block-reason small" id="rv-merge-reason">{live ? evidence.blockedReason?.replaceAll("simulated merge", "local acceptance").replaceAll("merging", "accepting") : evidence.blockedReason}</p>}
        {status === "merged" && <p className="small muted">Merge simulated locally{latestDecision?.type === "merge" ? ` at ${new Date(latestDecision.at).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })}` : ""} · no GitHub call.</p>}
        {status === "accepted" && <p className="small muted">Accepted locally · no merge, push, or GitHub call.</p>}
      </div>
      {!["merged", "accepted"].includes(status) && <div className="rv-decisions"><button type="button" className="btn" disabled={!evidence.sourceValid || busy || deciding} onClick={() => openDecision("changes")}>{status === "changes" ? "Edit request" : "Request changes"}</button>
        <button type="button" className="btn primary" disabled={!evidence.canMerge || busy || deciding} aria-describedby={!evidence.canMerge ? "rv-merge-reason" : undefined} onClick={() => openDecision("merge")}>{live ? "Accept locally" : "Merge preview"}</button></div>}
      {notice && <div className="rv-action-notice"><Notice text={notice} /></div>}
    </footer>
    <Dialog id="rv-changes" open={dialog?.type === "changes"} onClose={() => setDialog(null)} title={`Request changes on #${dialogReview?.number ?? ""}`}>
      <form onSubmit={(event) => { event.preventDefault(); confirmDecision(draft); }}>
        <p className="small muted rv-dialog-source">Attempt {dialog?.expectedSource.attempt} · <span className="mono">{dialog?.expectedSource.revision}</span></p>
        <label className="field-label" htmlFor="rv-change-text">What should change?</label>
        <textarea ref={feedback} id="rv-change-text" rows={5} value={draft} onChange={(event) => { const value = event.target.value; setState((previous) => ({ ...previous, drafts: { ...previous.drafts, [dialog.reviewId]: value } })); }} />
        <p className="help">{storeError ? "Draft kept in memory; local storage is unavailable." : "Draft saved locally, including when you cancel."} {live ? "Feedback is recorded locally on this revision. No GitHub comment is posted." : "Recording posts no comment and dispatches no agent."}</p>
        {decisionError && <p className="error" role="alert">{decisionError}</p>}
        <div className="dialog-foot rv-dialog-actions"><button type="button" className="btn" onClick={() => setDialog(null)}>Cancel</button><button type="submit" className="btn primary" disabled={!draft.trim() || busy || deciding}>Record request</button></div>
      </form>
    </Dialog>
    <Dialog id="rv-merge" open={dialog?.type === "merge"} onClose={() => setDialog(null)} title={`${live ? "Accept locally" : "Simulate merge of"} #${dialogReview?.number ?? ""}?`}
      footer={<><button type="button" className="btn" onClick={() => setDialog(null)}>Cancel</button><button type="button" className="btn primary" disabled={busy || deciding} onClick={() => confirmDecision()}>{live ? "Record local acceptance" : "Record simulated merge"}</button></>}>
      <p>{live ? "Accept this exact revision locally. This does not merge, publish, or mark coding work delivered." : "This records a local preview decision. Nothing is merged and no GitHub call is made."}</p>
      <p className="small muted">Run <span className="mono">{dialog?.expectedSource.runId}</span> · Attempt {dialog?.expectedSource.attempt} · <span className="mono">{dialog?.expectedSource.revision}</span></p>
      <p className="small">Current checks and this exact source are checked again when you confirm.</p>
      {decisionError && <p className="error" role="alert">{decisionError}</p>}
    </Dialog>
  </section>;
}

function PreviewPane({ review, zoom, onChanges }) {
  const [failed, setFailed] = useState(false);
  if (!review.preview || failed) return <div className="rv-empty rv-preview-empty"><ImageBroken size={24} aria-hidden="true" />
    <h2>{failed ? "Screenshot could not load" : "No visual preview"}</h2>
    <p className="muted">{failed ? "The artifact is unavailable. The changes and recorded evidence are still readable." : review.previewNote ?? "No screenshot was captured for this revision."}</p>
    <div className="rv-empty-actions">{failed && <button type="button" className="btn" onClick={() => setFailed(false)}>Try again</button>}
      <button type="button" className="btn" onClick={onChanges}>View changes <ArrowRight size={14} aria-hidden="true" /></button></div></div>;
  return <div className={`rv-preview-scroll rv-zoom-${zoom}`} tabIndex={0} aria-label={zoom === "actual" ? "Screenshot at native pixel size. Scroll to inspect." : "Screenshot fitted to useful reading width. Scroll to inspect."}>
    <figure className="rv-preview-canvas"><img src={review.preview.src} alt={review.preview.alt} width={1206} height={2622} onError={() => setFailed(true)} />
      <figcaption className="small muted">{review.live ? "Captured screenshot" : "Sample screenshot"}{zoom === "actual" ? " · native pixels" : ""}</figcaption></figure>
  </div>;
}

function DiffPane({ review, file, fileIndex, onFile }) {
  if (!file) return <div className="rv-empty"><h2>No changes available</h2><p className="muted">This review has no captured file diff.</p></div>;
  let line = 0;
  return <div className="rv-changes-pane"><div className="rv-file-tools"><label className="rv-file-picker"><span className="small muted">File</span>
    <select aria-label="Changed file" value={fileIndex} onChange={(event) => onFile(Number(event.target.value))}>{review.files.map((item, index) => <option key={item.path} value={index}>{item.path}</option>)}</select></label>
    <span className="rv-change-count mono"><span className="add">+{file.add}</span> <span className="del">−{file.del}</span></span></div>
    <p className="rv-file-path mono">{file.path}</p>
    <div className="rv-diff-scroll" tabIndex={0} aria-label={`Changes in ${file.path}`}><table className="rv-diff"><tbody>{(file.lines ?? []).map((item, index) => {
      if (item.t !== "del") line += 1;
      return <tr key={index} className={`rv-line-${item.t}`}><td className="rv-line-number mono" aria-hidden="true">{item.newLine ?? item.oldLine ?? (item.t === "del" || item.t === "hunk" ? "" : line)}</td>
        <td className="rv-line-mark mono" aria-hidden="true">{item.t === "add" ? "+" : item.t === "del" ? "−" : ""}</td>
        <td className="rv-code mono">{["add", "del"].includes(item.t) && <span className="sr-only">{item.t === "add" ? "Added: " : "Removed: "}</span>}{item.s || " "}</td></tr>;
    })}</tbody></table></div>
  </div>;
}

function EvidencePane({ review, evidence, requested, decisions }) {
  return <div className="rv-evidence-scroll"><div className="rv-evidence-doc">
    <section><h2>Source</h2><p className="muted small">Evidence belongs to this review's pinned source.</p>
      <dl className="rv-source-facts"><div><dt>Run</dt><dd className="mono">{review.runId ?? "Unavailable"}</dd></div><div><dt>Attempt</dt><dd>{review.attempt ?? "Unavailable"}</dd></div><div><dt>Revision</dt><dd className="mono">{review.revision ?? "Unavailable"}</dd></div>
        <div><dt>Model snapshot</dt><dd>{model(evidence.run?.settings?.model)?.name ?? evidence.run?.settings?.model ?? "Not captured"}</dd></div>
        {review.live && <><div><dt>Source observed</dt><dd>{review.sourceCurrent ? "Current and clean" : "Needs reconciliation"}</dd></div><div><dt>Observed head</dt><dd className="mono">{review.observedHead ?? "Not observed"}</dd></div></>}</dl>
      {!evidence.sourceValid && <p className="error">{evidence.blockedReason} Decisions are unavailable.</p>}
    </section>
    <section><h2>Checks <span className="muted small">{evidence.passed}/{evidence.total} passed on this revision</span></h2>
      {evidence.currentChecks.length || evidence.staleChecks.length ? <div className="rv-table-scroll" tabIndex={0} aria-label="Check evidence"><table className="rv-check-table"><thead><tr><th scope="col">Check</th><th scope="col">Result</th><th scope="col">Revision</th><th scope="col">Observed</th></tr></thead>
        <tbody>{[...evidence.currentChecks.map((item) => ({ ...item, stale: false })), ...evidence.staleChecks.map((item) => ({ ...item, stale: true }))].map((item, index) => <tr key={`${item.name}-${index}`}>
          <td>{item.name}{item.stale && <span className="rv-old-check small warn-text">{evidence.sourceCurrent ? "Older or unknown revision · not counted" : "Historical evidence · source is not current"}</span>}{review.live && <details className="small"><summary>Command and output</summary><p className="mono">{item.program} {(item.args ?? []).join(" ")}</p><p>Exit: {item.exitCode ?? "not observed"}{item.timedOut ? " · timed out" : ""}</p><pre className="live-json">{item.output || "No output captured."}</pre></details>}</td><td className={item.result === "pass" && !item.stale ? "ok" : item.result === "fail" ? "bad" : "muted"}>{RESULT_LABELS[item.result] ?? "Unknown"}</td><td className="mono">{item.revision ?? "Unknown"}</td><td className="mono muted">{item.at == null ? "—" : clock(item.at)}</td></tr>)}</tbody></table></div>
        : <p className="muted">No check evidence was captured.</p>}
    </section>
    <section><h2>Artifacts</h2>{evidence.artifacts.length ? <ul className="rv-artifacts">{evidence.artifacts.map((item, index) => <li key={`${item.label}-${index}`}><div><span>{item.label}</span><span className={`rv-artifact-state ${item.state === "current" ? "ok" : "muted"}`}>{item.state}</span></div>
      <p className="small muted"><span className="mono">{item.revision ?? "Revision unknown"}</span>{item.note ? ` · ${item.note}` : ""}</p>
      {safeArtifactHref(item.href) && <a className="text-link small" href={safeArtifactHref(item.href)} target="_blank" rel="noreferrer">Open artifact{item.state !== "current" ? " (not current evidence)" : ""}</a>}</li>)}</ul> : <p className="muted">No artifacts were captured.</p>}</section>
    <section><h2>Limitations</h2>{evidence.limitations.length ? <ul className="rv-limitations">{evidence.limitations.map((item) => <li key={item}><WarningCircle size={15} aria-hidden="true" /><span>{item}</span></li>)}</ul> : <p className="muted">No limitations recorded.</p>}</section>
    {(requested || decisions.length > 0) && <section><h2>Local decisions</h2>{requested && <p className="rv-feedback">Changes requested: {requested}</p>}
      {decisions.length > 0 ? <ol className="rv-decision-history">{decisions.map((item, index) => <li key={item.id ?? index}><span>{item.type === "accept" ? "Accepted locally" : item.type === "merge" ? "Merge simulated" : "Change request recorded"}</span>
        <span className="small muted">{new Date(item.at).toLocaleString()} · Attempt {item.attempt} · <span className="mono">{item.revision}</span></span>{item.text && <p>{item.text}</p>}</li>)}</ol> : <p className="small muted">Example feedback on the pinned source. No comment was posted.</p>}
    </section>}
  </div></div>;
}
