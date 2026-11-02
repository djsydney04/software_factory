import { model } from "./data.js";

const STATES = new Set(["ready", "changes", "merged"]);
const RESULTS = new Set(["pass", "fail", "running", "pending", "unknown"]);
const ARTIFACT_STATES = new Set(["current", "stale", "missing", "expired"]);
const record = (value) => value && typeof value === "object" && !Array.isArray(value);
const text = (value) => typeof value === "string" && value.trim().length > 0;
const attempt = (value) => Number.isInteger(value) && value > 0;
const sameSource = (a, b) => a?.runId === b?.runId && a?.attempt === b?.attempt && a?.revision === b?.revision;

export function reviewEvidence(review, runs, tasks, status = "ready") {
  const run = (Array.isArray(runs) ? runs : []).find((item) => record(item) && item.id === review?.runId) ?? null;
  const task = (Array.isArray(tasks) ? tasks : []).find((item) => record(item) && item.id === run?.taskId) ?? null;
  const sourceCurrent = !review?.live || review.sourceCurrent === true;
  const sourceValid = Boolean(sourceCurrent && run && task && text(review?.runId) && attempt(review?.attempt) && text(review?.revision)
    && run.attempt === review.attempt && run.revision === review.revision);
  const title = text(review?.title) ? review.title : text(task?.title) ? task.title : `Review #${review?.number ?? "unknown"}`;
  const provider = model(run?.settings?.model)?.provider ?? (run?.live ? "Codex" : "Unknown provider");
  const rawChecks = Array.isArray(run?.checks) ? run.checks : [];
  const malformedChecks = !Array.isArray(run?.checks) || rawChecks.some((check) => !record(check)
    || !text(check.name) || !text(check.revision) || !RESULTS.has(check.result));
  const checks = rawChecks.map((check) => ({
    ...(record(check) ? check : {}),
    name: text(check?.name) ? check.name : "Unknown check",
    revision: text(check?.revision) ? check.revision : null,
    result: RESULTS.has(check?.result) ? check.result : "unknown",
    at: Number.isFinite(check?.at) ? check.at : null,
  }));
  const currentChecks = checks.filter((check) => sourceCurrent && text(review?.revision) && check.revision === review.revision);
  const staleChecks = checks.filter((check) => !sourceCurrent || !text(review?.revision) || check.revision !== review.revision);
  const passed = currentChecks.filter((check) => check.result === "pass").length;
  const newerAttempt = (Array.isArray(runs) ? runs : []).some((item) => record(item) && item.taskId === run?.taskId
    && attempt(item.attempt) && item.attempt > review?.attempt);
  const artifacts = (Array.isArray(run?.artifacts) ? run.artifacts : []).filter(record).map((artifact) => ({
    ...artifact,
    label: text(artifact.label) ? artifact.label : "Unnamed artifact",
    revision: text(artifact.revision) ? artifact.revision : null,
    state: !ARTIFACT_STATES.has(artifact.state) ? "unknown"
      : artifact.state === "current" && (!sourceCurrent || artifact.revision !== review?.revision) ? "stale" : artifact.state,
  }));
  const limitations = [];
  if (text(review?.limitation)) limitations.push(review.limitation);
  if (!review?.preview && text(review?.previewNote)) limitations.push(review.previewNote);
  for (const artifact of artifacts) {
    if (artifact.state !== "current") limitations.push(`${artifact.label}: ${artifact.state}${text(artifact.note) ? ` — ${artifact.note}` : ""}.`);
  }
  let blockedReason = null;
  if (!sourceCurrent) blockedReason = review.sourceReason || "The service has not confirmed that this review source is current.";
  else if (!sourceValid) blockedReason = "The linked run, attempt, revision, or task is unavailable or has changed.";
  else if (status === "merged") blockedReason = "A simulated merge has already been recorded.";
  else if (status === "changes") blockedReason = "Requested changes are waiting for a new revision.";
  else if (status !== "ready") blockedReason = "This review is not ready for a simulated merge.";
  else if (run.status !== "succeeded") blockedReason = "The linked attempt must succeed before a simulated merge.";
  else if (newerAttempt) blockedReason = "A newer attempt exists for this task. Review its outcome before merging.";
  else if (malformedChecks) blockedReason = "Check evidence is unavailable or malformed.";
  else if (!currentChecks.length) blockedReason = "No checks certify this revision.";
  else if (passed !== currentChecks.length) blockedReason = "Every check on this revision must pass before a simulated merge.";
  return { run, task, title, provider, sourceValid, sourceCurrent, currentChecks, staleChecks, artifacts, passed,
    total: currentChecks.length, limitations: [...new Set(limitations)], canMerge: blockedReason === null, blockedReason };
}

export function restoreReviewState(saved, reviews) {
  const source = record(saved) ? saved : {};
  const savedStatus = record(source.status) ? source.status : {};
  const savedNotes = record(source.notes) ? source.notes : {};
  const savedDrafts = record(source.drafts) ? source.drafts : {};
  const known = (Array.isArray(reviews) ? reviews : []).filter((review) => record(review) && text(review.id));
  const status = {}, notes = {}, drafts = {};
  for (const review of known) {
    status[review.id] = STATES.has(savedStatus[review.id]) ? savedStatus[review.id]
      : STATES.has(review.group) ? review.group : "ready";
    if (typeof savedNotes[review.id] === "string") notes[review.id] = savedNotes[review.id];
    if (typeof savedDrafts[review.id] === "string") drafts[review.id] = savedDrafts[review.id];
  }
  const ids = new Set(known.map((review) => review.id));
  const decisions = (Array.isArray(source.decisions) ? source.decisions : []).filter((decision) => record(decision)
    && ids.has(decision.reviewId) && (decision.type === "merge" || decision.type === "changes")
    && text(decision.runId) && attempt(decision.attempt) && text(decision.revision) && Number.isFinite(decision.at)
    && (decision.type !== "changes" || text(decision.text)) && (decision.id === undefined || text(decision.id)))
    .map((decision) => ({ ...(decision.id === undefined ? {} : { id: decision.id }), reviewId: decision.reviewId,
      type: decision.type, runId: decision.runId, attempt: decision.attempt, revision: decision.revision,
      at: decision.at, ...(decision.type === "changes" ? { text: decision.text } : {}) }));
  return { status, notes, drafts, decisions };
}

export function recordReviewDecision(state, action, reviews, runs, tasks, { now = Date.now() } = {}) {
  const fail = (error) => ({ state, error });
  const review = (Array.isArray(reviews) ? reviews : []).find((item) => item?.id === action?.reviewId);
  if (!review || (action?.type !== "merge" && action?.type !== "changes")) return fail("Choose an available review action.");
  if (!sameSource(action.expectedSource, review)) return fail("The reviewed source changed. Reopen the decision for the current source.");
  const current = restoreReviewState(state, reviews);
  const evidence = reviewEvidence(review, runs, tasks, current.status[review.id]);
  if (!evidence.sourceValid) return fail(evidence.blockedReason);
  if (current.status[review.id] === "merged") return fail("A simulated merge has already been recorded.");
  if (!Number.isFinite(now)) return fail("A valid decision time is required.");
  if (action.type === "merge" && !evidence.canMerge) return fail(evidence.blockedReason);
  const message = typeof action.text === "string" ? action.text.trim() : "";
  if (action.type === "changes" && !message) return fail("Describe what should change.");
  if (action.type === "changes" && current.status[review.id] === "changes" && current.notes[review.id] === message)
    return fail("This change request has already been recorded.");
  const decision = { reviewId: review.id, type: action.type, runId: review.runId, attempt: review.attempt,
    revision: review.revision, at: now, ...(action.type === "changes" ? { text: message } : {}) };
  return { state: { ...current, status: { ...current.status, [review.id]: action.type === "merge" ? "merged" : "changes" },
    notes: action.type === "changes" ? { ...current.notes, [review.id]: message } : current.notes,
    drafts: action.type === "changes" ? { ...current.drafts, [review.id]: "" } : current.drafts,
    decisions: [...current.decisions, decision] }, error: null };
}
