import { formatRequestTime } from "./scheduling.js";

const ARRAYS = ["projects", "steering", "requests", "tasks", "runs", "reviews", "reviewSources", "approvals", "events", "leadWakes", "repositoryConnections"];
export const emptySnapshot = () => Object.fromEntries([...ARRAYS.map((key) => [key, []]), ["lastEventSequence", 0]]);

export function validateSnapshot(value) {
  if (value && value.repositoryConnections == null) value = { ...value, repositoryConnections: [] };
  if (!value || !ARRAYS.every((key) => Array.isArray(value[key])) || !Number.isSafeInteger(value.lastEventSequence)) {
    throw new Error("The service returned an unsupported snapshot. Reconnect after checking its version.");
  }
  return value;
}

export function observedCatalog(capabilities) {
  if (!capabilities?.available || !capabilities?.authenticated || !Array.isArray(capabilities.models)) return [];
  return capabilities.models.filter((model) => typeof model.id === "string" && Array.isArray(model.supportedReasoningEfforts))
    .map((model) => ({ id: model.id, provider: "Codex", name: model.displayName || model.id, available: true,
      efforts: model.supportedReasoningEfforts.map((item) => item.reasoningEffort).filter((effort) => typeof effort === "string"),
      defaultEffort: model.defaultReasoningEffort }));
}

export function validateLiveCombo(catalog, model, effort) {
  const selected = catalog.find((item) => item.id === model);
  if (!selected) return "Select a model from the connected Codex catalog in Settings.";
  return selected.efforts.includes(effort) ? null : "Choose an effort supported by the selected Codex model.";
}

export const liveDefaults = () => ({ model: "", effort: "medium", access: "worktree",
  permissions: { commands: "ask", network: "ask", integrations: "ask" }, maxTurnSeconds: 180,
  maxOutputTokens: 8000, maxAttempts: 1, reviewQueueCap: 3 });

export function liveScheduleLabel(request) {
  if (request.policy !== "scheduled") return ({ lead_decides: "Lead decides", next_available: "Next available" })[request.policy] ?? request.policy;
  const at = Date.parse(request.scheduledAt);
  if (!Number.isFinite(at)) return "Scheduled time unavailable";
  if (!request.timeZone) return `${request.scheduledAt} · time zone not captured`;
  try { return `${formatRequestTime(at, request.timeZone, true)} · ${request.timeZone}`; }
  catch { return `${request.scheduledAt} · saved time zone unavailable (${request.timeZone})`; }
}

export function parseUnifiedDiff(diff) {
  const files = [];
  let file = null, oldLine = null, newLine = null;
  for (const line of String(diff ?? "").split("\n")) {
    if (line.startsWith("diff --git ")) {
      file = { path: line.replace(/^diff --git a\/(.*?) b\//, ""), add: 0, del: 0, lines: [] };
      files.push(file); oldLine = null; newLine = null;
      continue;
    }
    if (!file) continue;
    if (line.startsWith("+++ ")) {
      if (line !== "+++ /dev/null") file.path = line.slice(4).replace(/^b\//, "");
      continue;
    }
    if (line.startsWith("--- ")) { if (line !== "--- /dev/null" && file.path.startsWith("diff --git")) file.path = line.slice(4).replace(/^a\//, ""); continue; }
    const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
    if (hunk) { oldLine = Number(hunk[1]); newLine = Number(hunk[2]); file.lines.push({ t: "hunk", s: line }); continue; }
    if (newLine != null && line.startsWith("+")) { file.add++; file.lines.push({ t: "add", s: line.slice(1), newLine: newLine++ }); }
    else if (oldLine != null && line.startsWith("-")) { file.del++; file.lines.push({ t: "del", s: line.slice(1), oldLine: oldLine++ }); }
    else if (newLine != null && line.startsWith(" ")) { file.lines.push({ t: "ctx", s: line.slice(1), newLine: newLine++, oldLine: oldLine++ }); }
    else if (line) file.lines.push({ t: "hunk", s: line });
  }
  if (!files.length && String(diff ?? "").trim()) files.push({ path: "Captured diff", add: 0, del: 0,
    lines: String(diff).split("\n").map((s) => ({ t: "hunk", s })) });
  return files;
}

const timestamp = (value) => Number.isFinite(Date.parse(value)) ? Date.parse(value) : null;
const checkView = (check) => ({ ...check, result: check.timedOut || (check.exitCode != null && check.exitCode !== 0) ? "fail" : check.exitCode === 0 ? "pass" : "unknown", at: timestamp(check.finishedAt) });
const stage = (status) => ["needs_approval", "recovery_required"].includes(status) ? "waiting" : status === "checking" ? "check" : status === "claimed" ? "plan" : "build";
const eventText = (event) => event.payload?.summary || event.payload?.reason || ({ run_claimed: "Attempt claimed by the supervisor.",
  run_recovery_required: "Execution needs reconciliation after restart.", run_updated: `Observed state: ${String(event.payload?.status ?? "updated").replaceAll("_", " ")}.` })[event.kind] || event.kind.replaceAll("_", " ");

export function buildLiveView(snapshot, projectId) {
  const project = snapshot.projects.find((item) => item.id === projectId) ?? snapshot.projects[0] ?? null;
  if (!project) return { project: null, tasks: [], runs: [], reviews: [], messages: [], requests: [], reviewState: { status: {}, notes: {}, decisions: [], drafts: {} } };
  const own = (items) => items.filter((item) => item.projectId === project.id);
  const tasks = own(snapshot.tasks).map((item) => ({ ...item, live: true, short: item.title }));
  const records = own(snapshot.reviews);
  const reviews = records.map((review, index) => {
    const source = snapshot.reviewSources.find((item) => item.reviewId === review.id);
    const sourceCurrent = source?.sourceCurrent === true && source.observedHead === review.revision;
    return { ...review, live: true, number: index + 1, sourceCurrent, observedHead: source?.observedHead ?? null,
    sourceReason: sourceCurrent ? null : source?.reason || (source ? "The worktree source is no longer current." : "The service has not reconciled this review source."),
    title: tasks.find((task) => task.id === review.taskId)?.title ?? "Local review",
    summary: "Exact worktree diff and supervisor-observed checks.", group: review.decision === "accepted_local" ? "accepted" : review.decision === "changes_requested" ? "changes" : sourceCurrent ? "ready" : "blocked",
    preview: null, previewNote: "No screenshot was captured for this revision.", limitation: review.limitations.join(" ") || null,
    files: parseUnifiedDiff(review.diff) };
  });
  const events = own(snapshot.events);
  const runs = own(snapshot.runs).map((run) => {
    const activity = events.filter((event) => event.runId === run.id);
    const review = reviews.find((item) => item.runId === run.id && item.revision === run.headSha);
    const approval = own(snapshot.approvals).find((item) => item.runId === run.id && item.decision == null);
    const startedAt = timestamp(run.startedAt);
    const stages = activity.filter((event) => ["run_updated", "run_claimed", "run_recovery_required"].includes(event.kind))
      .map((event) => ({ stage: stage(event.payload?.status ?? (event.kind === "run_recovery_required" ? "recovery_required" : "claimed")), at: timestamp(event.createdAt) }));
    return { ...run, live: true, executionSettings: run.authorization?.settings ?? null,
      reviewSourceCurrent: review?.sourceCurrent ?? null, reviewSourceReason: review?.sourceReason ?? null, observedHead: review?.observedHead ?? null,
      settings: { ...(run.authorization?.settings ?? run.effectiveSettings), mode: "bounded", source: { model: "captured", effort: "captured", mode: "captured" },
      contextRevision: String(run.contextRevision), contextFiles: [], capturedAt: startedAt },
      revision: run.headSha, startedAt, updatedAt: timestamp(activity.at(-1)?.createdAt) ?? startedAt, endedAt: timestamp(run.finishedAt),
      stages: stages.length ? stages : [{ stage: stage(run.status), at: startedAt }],
      activity: activity.map((event) => ({ at: timestamp(event.createdAt), text: eventText(event) })).filter((item,index,all)=>!index||item.text!==all[index-1].text),
      failure: run.reason, changed: review?.files.map(({ path, add, del }) => ({ path, add, del })) ?? [],
      checks: (review?.checks ?? run.checks).map(checkView), reviewId: review?.id,
      artifacts: (review?.artifacts ?? []).map((item) => ({ label: item.title, kind: item.kind, state: review?.sourceCurrent && item.revision === run.headSha ? "current" : "stale", revision: item.revision, note: item.path })),
      approval: approval ? { id: approval.id, request: approval.detail, scope: approval.scope, short: approval.scope, requestedAt: timestamp(approval.createdAt) } : null };
  });
  const messages = [];
  for (const steering of own(snapshot.steering)) {
    const attemptCaptured = runs.some((run) => run.context?.steering?.some((item) => item.id === steering.id));
    const leadCaptured = own(snapshot.leadWakes).some((wake)=>wake.status==="completed"&&wake.context?.steering?.some((item)=>item.id===steering.id));
    messages.push({ id: steering.id, role: "you", sentAt: timestamp(steering.createdAt), at: timestamp(steering.createdAt), text: steering.text });
    messages.push({ id: `${steering.id}-receipt`, role: "receipt", at: timestamp(steering.createdAt), text: `Steering received · revision ${steering.revision}${attemptCaptured ? " · captured by an attempt" : leadCaptured ? " · captured by the lead" : " · awaiting the next planning turn"}.` });
  }
  const requests = own(snapshot.requests);
  for (const request of requests) {
    messages.push({ id: request.id, role: "you", requestId: request.id, sentAt: timestamp(request.createdAt), at: timestamp(request.createdAt), text: request.text,
      requestMeta: `One-off · ${liveScheduleLabel(request)}` });
    messages.push({ id: `${request.id}-receipt`, role: "receipt", requestId: request.id, at: timestamp(request.createdAt), text: `Received by the service · ${request.status.replaceAll("_", " ")}.` });
    if (request.planningSummary || request.reason) messages.push({ id: `${request.id}-lead`, role: "lead", at: timestamp(events.findLast((event) => event.payload?.requestId === request.id)?.createdAt) ?? timestamp(request.createdAt), text: request.planningSummary || request.reason });
  }
  for (const review of reviews) messages.push({ id: `${review.id}-lead`, role: "lead", at: timestamp(review.createdAt),
    text: review.sourceCurrent ? "Work is available for local review." : "Captured work is available. Its source must be reconciled before a decision.", reviewId: review.id });
  for (const wake of own(snapshot.leadWakes).filter((item)=>["completed","failed","waiting","recovery_required"].includes(item.status))) {
    messages.push({id:`${wake.id}-lead`,role:"lead",at:timestamp(wake.finishedAt??wake.createdAt),text:wake.summary||wake.reason});
  }
  for (const approval of own(snapshot.approvals).filter((item) => item.decision == null)) messages.push({ id: `${approval.id}-lead`, role: "lead", at: timestamp(approval.createdAt), text: approval.detail, approval: approval.runId });
  messages.sort((a, b) => a.at - b.at);
  const reviewState = { status: Object.fromEntries(reviews.map((item) => [item.id, item.group])), notes: Object.fromEntries(reviews.filter((item) => item.feedback).map((item) => [item.id, item.feedback])), drafts: {},
    decisions: events.filter((event) => event.kind === "review_decided_local").map((event) => ({ id: String(event.sequence), reviewId: event.payload.reviewId,
      type: event.payload.decision === "accepted_local" ? "accept" : "changes", runId: event.runId, attempt: reviews.find((item) => item.id === event.payload.reviewId)?.attempt,
      revision: event.payload.revision, at: timestamp(event.createdAt), text: event.payload.feedback })) };
  return { project, tasks, runs, reviews, messages, requests, reviewState };
}
