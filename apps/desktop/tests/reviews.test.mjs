import assert from "node:assert/strict";
import test from "node:test";
import { reviewEvidence, restoreReviewState, recordReviewDecision } from "../src/review-state.js";

const review = { id: "RV-1", number: 1, title: "Review the layout", runId: "R-1", attempt: 2, revision: "new", group: "ready", preview: null, previewNote: "Text-only change." };
const task = { id: "T-1", title: "Source task" };
const run = { id: "R-1", taskId: "T-1", attempt: 2, status: "succeeded", revision: "new", settings: { model: "codex-gpt-5.3" },
  checks: [{ name: "Build", result: "pass", revision: "new", at: 12 }], artifacts: [] };
const source = { runId: "R-1", attempt: 2, revision: "new" };
const initial = () => restoreReviewState(null, [review]);
const evidence = (changedRun = run, changedReview = review, state = "ready") => reviewEvidence(changedReview, [changedRun], [task], state);
const decide = (state, action, changedRun = run, changedReview = review) => recordReviewDecision(state,
  { reviewId: "RV-1", expectedSource: source, ...action }, [changedReview], [changedRun], [task], { now: 1_800_000_000_000 });

test("only exact revision checks certify the review; stale green evidence cannot enable merge", () => {
  const result = evidence({ ...run, checks: [{ name: "Build", result: "pass", revision: "old" }, { name: "UI tests", result: "running", revision: "new" }] });
  assert.equal(result.passed, 0);
  assert.equal(result.total, 1);
  assert.equal(result.staleChecks[0].revision, "old");
  assert.equal(result.canMerge, false);
  assert.equal(evidence({ ...run, checks: [{ name: "Build", result: "pass", revision: "old" }] }).canMerge, false);
});

test("failed, pending, unknown, missing, and malformed checks block merge rather than looking green", () => {
  for (const checks of [[], null, {}, [null], [{ name: "Build", result: "pass" }], [{ name: "Build", result: "PASS", revision: "new" }],
    ...["fail", "pending", "unknown", "running"].map((result) => [...run.checks, { name: "UI tests", result, revision: "new" }])]) {
    const result = evidence({ ...run, checks });
    assert.equal(result.canMerge, false, JSON.stringify(checks));
    assert.equal(typeof result.blockedReason, "string");
  }
});

test("missing optional previews and stale artifacts remain visible limitations without blocking text-only review", () => {
  const result = evidence({ ...run, artifacts: [{ label: "Screenshot", state: "missing", revision: "new" },
    { label: "Before", state: "current", revision: "old", href: "/before.png" }] });
  assert.equal(result.canMerge, true);
  assert.equal(result.passed, 1);
  assert.equal(result.artifacts[1].state, "stale");
  assert.equal(result.artifacts[1].revision, "old");
  assert.ok(result.limitations.some((item) => item.includes("Screenshot: missing")));
  assert.ok(result.limitations.includes("Text-only change."));
});

test("missing run, task, pinned source, or changed source safely disables decisions", () => {
  for (const result of [reviewEvidence(review, [], [task]), reviewEvidence(review, [run], []),
    evidence(run, { ...review, revision: undefined }), evidence(run, { ...review, attempt: undefined }),
    evidence({ ...run, revision: "later" }), evidence({ ...run, attempt: 3 })]) {
    assert.equal(result.sourceValid, false);
    assert.equal(result.canMerge, false);
    assert.equal(result.title, "Review the layout");
  }
  assert.equal(reviewEvidence(null, [run], [task]).title, "Review #unknown");
});

test("confirmation rejects source changes and ineligible checks atomically", () => {
  const state = initial();
  for (const result of [decide(state, { type: "merge" }, { ...run, revision: "later" }),
    decide(state, { type: "changes", text: "Fix spacing" }, run, { ...review, revision: "later" }),
    decide(state, { type: "merge", expectedSource: { ...source, attempt: 1 } }),
    decide(state, { type: "merge" }, { ...run, checks: [] })]) {
    assert.equal(result.state, state);
    assert.equal(typeof result.error, "string");
    assert.deepEqual(state.decisions, []);
  }
});

test("merge requires a successful latest task attempt and rechecks supersession at confirmation", () => {
  assert.equal(evidence().canMerge, true);
  for (const status of ["running", "cancelled", "failed"]) {
    const result = evidence({ ...run, status });
    assert.equal(result.sourceValid, true);
    assert.equal(result.canMerge, false);
    assert.match(result.blockedReason, /must succeed/);
  }
  const state = initial();
  for (const status of ["queued", "succeeded"]) {
    const newer = { ...run, id: "R-NEWER", attempt: 3, revision: "later", status };
    const result = reviewEvidence(review, [run, newer], [task]);
    assert.equal(result.sourceValid, true);
    assert.equal(result.canMerge, false);
    assert.match(result.blockedReason, /newer attempt/);
    const confirmation = recordReviewDecision(state, { reviewId: "RV-1", type: "merge", expectedSource: source },
      [review], [run, newer], [task], { now: 1_800_000_000_000 });
    assert.equal(confirmation.state, state);
    assert.match(confirmation.error, /newer attempt/);
  }
  assert.equal(reviewEvidence(review, [run, { ...run, id: "R-OLDER", attempt: 1, status: "failed" }], [task]).canMerge, true);
  assert.equal(reviewEvidence(review, [run, { ...run, id: "R-OTHER", taskId: "T-OTHER", attempt: 3 }], [task]).canMerge, true);
});

test("restoration retains independent per-review drafts and historical decision source without filling defaults", () => {
  const other = { ...review, id: "RV-2", group: "changes" };
  const saved = { status: { "RV-1": "invalid", "RV-2": "merged", GONE: "merged" },
    notes: { "RV-1": false, "RV-2": "Requested earlier" }, drafts: { "RV-1": "  Unfinished feedback\n", "RV-2": "Second draft", GONE: "unrelated" },
    decisions: [null, { reviewId: "RV-1", type: "merge", runId: "R-OLD", attempt: 1, revision: "old", at: 10 },
      { reviewId: "RV-2", type: "changes", runId: "R-1", attempt: 2, revision: "new", at: 11 },
      { reviewId: "RV-1", type: "merge", at: 12 }] };
  const restored = restoreReviewState(JSON.parse(JSON.stringify(saved)), [review, other]);
  assert.deepEqual(restored.status, { "RV-1": "ready", "RV-2": "merged" });
  assert.deepEqual(restored.drafts, { "RV-1": "  Unfinished feedback\n", "RV-2": "Second draft" });
  assert.deepEqual(restored.notes, { "RV-2": "Requested earlier" });
  assert.deepEqual(restored.decisions, [{ reviewId: "RV-1", type: "merge", runId: "R-OLD", attempt: 1, revision: "old", at: 10 }]);
  saved.decisions[1].revision = "mutated";
  assert.equal(restored.decisions[0].revision, "old");
});

test("requested changes save an immutable source snapshot and clear only their own draft", () => {
  const state = { ...initial(), drafts: { "RV-1": "Fix spacing", "RV-2": "Other feedback" } };
  const before = structuredClone(run);
  const result = recordReviewDecision(state, { reviewId: "RV-1", expectedSource: source, type: "changes", text: "  Fix spacing  " },
    [review, { ...review, id: "RV-2" }], [run], [task], { now: 1_800_000_000_000 });
  assert.equal(result.error, null);
  assert.equal(result.state.status["RV-1"], "changes");
  assert.equal(result.state.notes["RV-1"], "Fix spacing");
  assert.equal(result.state.drafts["RV-1"], "");
  assert.equal(result.state.drafts["RV-2"], "Other feedback");
  assert.equal(state.drafts["RV-1"], "Fix spacing");
  assert.deepEqual(result.state.decisions[0], { reviewId: "RV-1", type: "changes", ...source, at: 1_800_000_000_000, text: "Fix spacing" });
  assert.deepEqual(run, before);
  assert.equal(decide(result.state, { type: "changes", text: "Fix spacing" }).state, result.state);
  assert.equal(decide(result.state, { type: "merge" }).state, result.state);
});

test("simulated merge is recorded once and never mutates a run or reuses a merged review", () => {
  const state = initial();
  const before = structuredClone(run);
  const result = decide(state, { type: "merge" });
  assert.equal(result.error, null);
  assert.equal(result.state.status["RV-1"], "merged");
  assert.equal(result.state.decisions.length, 1);
  assert.equal(state.decisions.length, 0);
  assert.deepEqual(run, before);
  for (const action of [{ type: "merge" }, { type: "changes", text: "Another request" }]) {
    const repeated = decide(result.state, action);
    assert.equal(repeated.state, result.state);
    assert.equal(repeated.state.decisions.length, 1);
    assert.match(repeated.error, /already/);
  }
});
