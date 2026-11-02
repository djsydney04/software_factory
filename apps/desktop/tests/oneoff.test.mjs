import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import test from "node:test";
import { createRequest, cancelRequest, rescheduleRequest, requestState, restoreRequests, validateTiming, localInputValue, policyLabel } from "../src/scheduling.js";

const NOW = Date.UTC(2027, 0, 1, 12);
const settings = { model: "codex-gpt-5.3", effort: "medium", mode: "ask", access: "worktree", source: { model: "default" } };
const context = { contextFiles: ["AGENTS.md", "src/main.rs"], roots: ["src/"], attached: ["AGENTS.md"], ignore: ".env*" };
const make = (changes = {}) => createRequest({ id: "REQ-TEST", text: "Audit the queue", policy: "lead", localTime: "", settings, context, now: NOW, ...changes });

test("one-off timing requires a future real local time and keeps untimed policies untimed", () => {
  assert.match(validateTiming("at", "", NOW).error, /Choose a future/);
  assert.match(validateTiming("at", "not-a-date", NOW).error, /valid local/);
  assert.match(validateTiming("at", "2027-02-30T09:00", NOW).error, /does not exist/);
  assert.match(validateTiming("at", localInputValue(NOW), NOW).error, /future/);
  const future = validateTiming("at", localInputValue(NOW + 86_400_000), NOW);
  assert.equal(future.error, null);
  assert.equal(future.scheduledAt, NOW + 86_400_000);
  assert.equal(validateTiming("next", "old invalid field", NOW).scheduledAt, null);
  assert.match(make({ text: "  " }).error, /Describe/);
  assert.match(make({ settings: { ...settings, model: "gemini-3.1" } }).error, /not connected/);
  const scheduled = make({ policy: "at", localTime: "2027-11-01T10:00" }).request;
  assert.match(policyLabel(scheduled), /2027/);
  assert.match(policyLabel({ ...scheduled, scheduledAt: new Date(2028, 10, 1, 10).getTime() }), /2028/);
});

test("DST gaps are rejected and ambiguous times retain their resolved absolute offset", () => {
  const moduleUrl = new URL("../src/scheduling.js", import.meta.url).href;
  const result = JSON.parse(execFileSync(process.execPath, ["--input-type=module", "-e", `
    import { validateTiming, formatRequestTime } from ${JSON.stringify(moduleUrl)};
    const now = Date.UTC(2027, 0, 1);
    const gap = validateTiming('at', '2027-03-14T02:30', now);
    const overlap = validateTiming('at', '2027-11-07T01:30', now);
    console.log(JSON.stringify({ gap, overlap, display: formatRequestTime(overlap.scheduledAt, overlap.timeZone) }));
  `], { env: { ...process.env, TZ: "America/Los_Angeles" }, encoding: "utf8" }));
  assert.match(result.gap.error, /does not exist/);
  assert.equal(result.overlap.scheduledAt, Date.UTC(2027, 10, 7, 8, 30));
  assert.equal(result.overlap.timeZone, "America/Los_Angeles");
  assert.match(result.display, /GMT-7/);
});

test("submission captures requested model and context independently of later defaults or attachments", () => {
  const cfg = structuredClone(settings);
  const inputs = structuredClone(context);
  const { request } = make({ settings: cfg, context: inputs });
  cfg.model = "claude-opus-5.5";
  cfg.source.model = "override";
  inputs.contextFiles.push(".env");
  inputs.roots.length = 0;
  assert.equal(request.settings.model, "codex-gpt-5.3");
  assert.equal(request.settings.source.model, "default");
  assert.deepEqual(request.context.contextFiles, ["AGENTS.md", "src/main.rs"]);
  assert.deepEqual(request.context.roots, ["src/"]);
  assert.equal(request.settings.capturedAt, NOW);
});

test("reschedule and cancellation preserve request identity and captured settings; cancelled intent stays cancelled", () => {
  const original = make().request;
  const scheduled = rescheduleRequest(original, "at", localInputValue(NOW + 86_400_000), NOW).request;
  assert.equal(scheduled.id, original.id);
  assert.equal(scheduled.createdAt, original.createdAt);
  assert.deepEqual(scheduled.settings, original.settings);
  assert.deepEqual(scheduled.context, original.context);
  assert.equal(original.status, "pending");
  assert.equal(scheduled.status, "scheduled");
  const cancelled = cancelRequest(scheduled, NOW + 1_000);
  assert.equal(cancelled.status, "cancelled");
  assert.deepEqual(cancelled.settings, original.settings);
  assert.deepEqual(cancelled.context, original.context);
  assert.match(rescheduleRequest(cancelled, "next", "", NOW).error, /cannot be rescheduled/);
  assert.equal(requestState(cancelled, NOW + 172_800_000).id, "cancelled");
});

test("passing the requested time derives Due without mutating intent or creating execution evidence", () => {
  const request = make({ policy: "at", localTime: localInputValue(NOW + 60_000) }).request;
  const before = JSON.stringify(request);
  assert.equal(requestState(request, NOW).id, "scheduled");
  assert.deepEqual(requestState(request, NOW + 60_000), { id: "due", label: "Due · awaiting lead" });
  assert.equal(JSON.stringify(request), before);
  assert.equal(request.runId, undefined);
  assert.equal(request.taskId, undefined);
});

test("request restore is backwards compatible and rejects malformed dates, context, zones, and duplicate identities", () => {
  const valid = make().request;
  assert.deepEqual(restoreRequests(undefined), []);
  const stored = JSON.parse(JSON.stringify(valid));
  const restored = restoreRequests([null, {}, { ...stored, id: "REQ-BAD-DATE", createdAt: 1e100 }, { ...stored, id: "REQ-BAD-TIME", policy: "at", scheduledAt: 1e100 }, { ...stored, id: "REQ-BAD-ZONE", timeZone: "nowhere" }, { ...stored, id: "REQ-BAD-CONTEXT", context: { contextFiles: [], roots: 4 } }, { ...stored, id: "REQ-BAD-IGNORE", context: { contextFiles: [], ignore: {} } }, { ...stored, id: "REQ-BAD-SETTINGS", settings: { ...settings, concurrency: {} } }, stored, stored]);
  assert.deepEqual(restored, [valid]);
  restored[0].context.contextFiles.push("other.md");
  assert.deepEqual(valid.context.contextFiles, ["AGENTS.md", "src/main.rs"]);
});
