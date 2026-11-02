import test from "node:test";
import assert from "node:assert/strict";
import { buildLiveView, emptySnapshot, liveScheduleLabel, observedCatalog, parseUnifiedDiff, validateLiveCombo, validateSnapshot } from "../src/live-state.js";
import { reviewEvidence } from "../src/review-state.js";
import { taskState } from "../src/data.js";

const time = "2026-11-01T23:01:00Z";
const source = () => ({ ...emptySnapshot(), projects: [{ id: "p", name: "Fixture", objective: "A real local goal", settings: {} }],
  tasks: [{ id: "t", projectId: "p", requestId: "q", title: "Change a greeting", status: "review" }],
  requests: [{ id: "q", projectId: "p", text: "Improve greeting", policy: "lead_decides", status: "planned", createdAt: time }],
  runs: [{ id: "r", projectId: "p", taskId: "t", requestId: "q", attempt: 1, status: "succeeded", headSha: "abc", contextRevision: 3,
    context: { steering: [{ id: "s" }] }, requestedSettings: { model: "observed-model" }, effectiveSettings: { model: "observed-model" }, checks: [], startedAt: time, finishedAt: time }],
  reviews: [{ id: "v", projectId: "p", taskId: "t", runId: "r", attempt: 1, revision: "abc", checks: [{ name: "test", program: "node", args: ["test.js"], revision: "abc", exitCode: 0, timedOut: false, output: "ok", finishedAt: time }], artifacts: [], limitations: [], diff: "", createdAt: time }],
  reviewSources: [{ reviewId: "v", sourceCurrent: true, observedHead: "abc", reason: null }] });

test("native state starts empty and rejects incompatible snapshots instead of loading fixtures", () => {
  const value = validateSnapshot(emptySnapshot());
  const view = buildLiveView(value);
  assert.equal(view.project, null);
  assert.deepEqual(view.runs, []);
  assert.deepEqual(view.reviews, []);
  assert.throws(() => validateSnapshot({ projects: [] }), /unsupported snapshot/);
});

test("request intent never becomes a fabricated run and project records stay separated", () => {
  const data = source();
  data.runs = []; data.reviews = [];
  data.requests.push({ ...data.requests[0], id: "other", projectId: "different" });
  const view = buildLiveView(data, "p");
  assert.equal(view.requests.length, 1);
  assert.equal(view.runs.length, 0);
  assert.equal(view.messages[1].role, "receipt");
  assert.match(view.messages[1].text, /Received by the service/);
});

test("only authenticated observed models and exact supported efforts are selectable", () => {
  const caps = { available: true, authenticated: true, models: [{ id: "actual", displayName: "Actual", defaultReasoningEffort: "high", supportedReasoningEfforts: [{ reasoningEffort: "high" }] }] };
  assert.deepEqual(observedCatalog({ ...caps, authenticated: false }), []);
  const catalog = observedCatalog(caps);
  assert.equal(validateLiveCombo(catalog, "actual", "high"), null);
  assert.match(validateLiveCombo(catalog, "actual", "medium"), /supported/);
  assert.match(validateLiveCombo(catalog, "fixture-model", "high"), /connected Codex catalog/);
});

test("actual exit codes certify only their pinned revision and local acceptance does not deliver a task", () => {
  const data = source();
  let view = buildLiveView(data, "p");
  assert.equal(reviewEvidence(view.reviews[0], view.runs, view.tasks).canMerge, true);
  data.reviews[0].checks[0].exitCode = null;
  view = buildLiveView(data, "p");
  assert.equal(view.runs[0].checks[0].result, "unknown");
  assert.equal(reviewEvidence(view.reviews[0], view.runs, view.tasks).canMerge, false);
  data.reviews[0].checks[0].exitCode = 0;
  data.reviews[0].checks[0].revision = "older";
  data.reviews[0].decision = "accepted_local";
  view = buildLiveView(data, "p");
  assert.equal(reviewEvidence(view.reviews[0], view.runs, view.tasks).passed, 0);
  assert.equal(view.reviewState.status.v, "accepted");
  assert.equal(taskState(view.tasks[0], view.runs, view.reviewState.status), "Review");
});

test("changed, dirty, or unobserved live sources block decisions without rewriting historical evidence", () => {
  for (const observedHead of ["abc", "changed", null]) {
    const data = source();
    data.reviewSources = [{ reviewId: "v", sourceCurrent: false, observedHead, reason: "The worktree changed after checks." }];
    const view = buildLiveView(data, "p");
    const evidence = reviewEvidence(view.reviews[0], view.runs, view.tasks);
    assert.equal(view.reviewState.status.v, "blocked");
    assert.equal(evidence.sourceValid, false);
    assert.equal(evidence.canMerge, false);
    assert.equal(evidence.passed, 0);
    assert.deepEqual(evidence.currentChecks, []);
    assert.equal(evidence.staleChecks[0].revision, "abc");
    assert.equal(evidence.staleChecks[0].result, "pass");
    assert.equal(view.runs[0].status, "succeeded");
    assert.equal(view.runs[0].headSha, "abc");
    assert.equal(view.reviews[0].revision, "abc");
    assert.match(evidence.blockedReason, /worktree changed/);
    assert.match(view.messages.find((item) => item.id === "v-lead").text, /reconciled/);
  }
  const missing = source();
  missing.reviewSources = [];
  const view = buildLiveView(missing, "p");
  assert.equal(reviewEvidence(view.reviews[0], view.runs, view.tasks).canMerge, false);
  assert.match(view.reviews[0].sourceReason, /has not reconciled/);
  const mismatched = source();
  mismatched.reviewSources[0].observedHead = "different";
  const mismatchedView = buildLiveView(mismatched, "p");
  assert.equal(reviewEvidence(mismatchedView.reviews[0], mismatchedView.runs, mismatchedView.tasks).canMerge, false);
});

test("execution settings use captured authorization while original permission ceilings remain historical", () => {
  const data = source();
  data.runs[0].effectiveSettings = { model: "observed-model", permissions: { commands: "ask", network: "ask", integrations: "ask" } };
  data.runs[0].authorization = { settings: { model: "observed-model", permissions: { commands: "allow", network: "deny", integrations: "deny" } },
    approvalIds: ["approval-1"], limitations: ["Network denied for this attempt."], authorizedAt: time };
  const view = buildLiveView(data, "p");
  assert.equal(view.runs[0].settings.permissions.commands, "allow");
  assert.equal(view.runs[0].settings.permissions.network, "deny");
  assert.equal(view.runs[0].effectiveSettings.permissions.network, "ask");
  assert.deepEqual(view.runs[0].authorization.approvalIds, ["approval-1"]);
  assert.deepEqual(view.runs[0].executionSettings, data.runs[0].authorization.settings);
  assert.equal(data.runs[0].effectiveSettings.permissions.network, "ask");
});

test("scheduled requests retain their saved zone across client zone changes and never guess missing zones", () => {
  const request = { policy: "scheduled", scheduledAt: "2026-06-01T16:00:00Z", timeZone: "America/Los_Angeles" };
  const previous = process.env.TZ;
  try {
    process.env.TZ = "Asia/Tokyo";
    const first = liveScheduleLabel(request);
    process.env.TZ = "Pacific/Honolulu";
    assert.equal(liveScheduleLabel(request), first);
    assert.match(first, /America\/Los_Angeles/);
    assert.match(first, /9:00/);
  } finally {
    if (previous == null) delete process.env.TZ; else process.env.TZ = previous;
  }
  assert.match(liveScheduleLabel({ ...request, timeZone: null }), /2026-06-01T16:00:00Z · time zone not captured/);
  assert.match(liveScheduleLabel({ ...request, timeZone: "Invalid/Zone" }), /saved time zone unavailable/);
});

test("steering receipt claims capture only when that exact steering exists in attempt context", () => {
  const data = source();
  data.steering = [{ id: "s", projectId: "p", revision: 2, text: "Keep it small", createdAt: time }];
  let view = buildLiveView(data, "p");
  assert.match(view.messages.find((item) => item.id === "s-receipt").text, /captured by an attempt/);
  data.runs[0].context = { steering: [] };
  view = buildLiveView(data, "p");
  assert.match(view.messages.find((item) => item.id === "s-receipt").text, /awaiting/);
});

test("a completed read-only wake acknowledges captured steering without fabricating an attempt", () => {
  const data = source(); data.runs = []; data.reviews = []; data.tasks = []; data.requests = [];
  data.steering = [{ id: "s", projectId: "p", revision: 2, text: "Keep it small", createdAt: time }];
  data.leadWakes = [{ id: "wake", projectId: "p", status: "running", context: { steering: [{ id: "s" }] }, createdAt: time }];
  let view = buildLiveView(data, "p");
  assert.match(view.messages.find((item) => item.id === "s-receipt").text, /awaiting/);
  assert.equal(view.messages.some((item) => item.id === "wake-lead"), false);
  data.leadWakes[0] = { ...data.leadWakes[0], status: "completed", summary: "I’ll use small verified steps.", finishedAt: time };
  view = buildLiveView(data, "p");
  assert.match(view.messages.find((item) => item.id === "s-receipt").text, /captured by the lead/);
  assert.equal(view.messages.find((item) => item.id === "wake-lead").text, "I’ll use small verified steps.");
  assert.equal(view.runs.length, 0); assert.equal(view.tasks.length, 0);
});

test("unified diffs preserve hunk coordinates, file identity and actual addition counts", () => {
  const files = parseUnifiedDiff("diff --git a/greet.js b/greet.js\nindex aa..bb 100644\n--- a/greet.js\n+++ b/greet.js\n@@ -8,2 +8,3 @@\n keep\n-old\n+new\n+extra\n");
  assert.equal(files[0].path, "greet.js");
  assert.equal(files[0].add, 2);
  assert.equal(files[0].del, 1);
  assert.deepEqual(files[0].lines.filter((line) => line.t === "add").map((line) => line.newLine), [9, 10]);
  assert.equal(files[0].lines.find((line) => line.t === "del").oldLine, 9);
});
