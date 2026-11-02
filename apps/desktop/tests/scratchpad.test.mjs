import assert from "node:assert/strict";
import test from "node:test";
import { archiveNote, createNote, defaultReviewConfig, editNote, previewTriage, restoreScratchpad, reviewCron, unarchiveNote, validateReviewConfig } from "../src/scratchpad.js";
import { cancelRequest } from "../src/scheduling.js";

const NOW = 1_800_000_000_000;
const project = { model: "codex-gpt-5.3", effort: "medium", mode: "ask", access: "read-only", permissions: { commands: "deny", network: "ask", integrations: "deny" } };
const context = { contextFiles: ["AGENTS.md"], roots: ["src/"], attached: [] };
const intake = () => ({ requests: [], scratchpad: { ...restoreScratchpad(null), notes: [createNote("NOTE-A", "Inspect the queue", NOW), createNote("NOTE-B", "Investigate the carousel", NOW)] } });

test("notes save explicit revisions and can archive and restore without losing sent provenance", () => {
  assert.equal(createNote("NOTE-EMPTY", "  ", NOW), null);
  const initial = createNote("NOTE-A", "  Fix the queue  ", NOW);
  const edited = editNote(initial, "Fix the queue and report back", NOW + 1_000);
  assert.equal(edited.revision, 2);
  assert.equal(initial.text, "Fix the queue");
  assert.equal(editNote(edited, "  ", NOW + 2_000), edited);
  assert.equal(unarchiveNote(archiveNote(edited)).status, "inbox");
  const sent = { ...edited, status: "sent", requestId: "REQ-ONE" };
  assert.equal(editNote(sent, "Rewrite sent history"), sent);
  assert.equal(unarchiveNote(archiveNote(sent)).status, "sent");
  assert.equal(unarchiveNote(archiveNote(sent)).requestId, "REQ-ONE");
});

test("morning review defaults to 09:00 and validates configurable cadence, local time, and zone", () => {
  const daily = { enabled: true, cadence: "daily", time: "09:00", timeZone: "America/Los_Angeles" };
  assert.equal(defaultReviewConfig().time, "09:00");
  assert.equal(defaultReviewConfig().cadence, "daily");
  assert.equal(validateReviewConfig(daily), null);
  assert.equal(reviewCron(daily), "0 9 * * *");
  assert.equal(reviewCron({ ...daily, time: "07:35", enabled: false }), "35 7 * * *");
  assert.equal(reviewCron({ ...daily, cadence: "hourly" }), "0 * * * *");
  assert.equal(reviewCron({ ...daily, cadence: "quarter-hour" }), "*/15 * * * *");
  assert.equal(reviewCron({ ...daily, cadence: "hourly", time: "" }), "0 * * * *");
  assert.equal(reviewCron({ ...daily, cadence: "quarter-hour", time: "" }), "*/15 * * * *");
  for (const bad of [{ cadence: "weekly" }, { time: "24:00" }, { time: "9:00" }, { timeZone: "Moon/Base" }, { timeZone: "+01:00" }, { enabled: "yes" }]) {
    assert.ok(validateReviewConfig({ ...daily, ...bad }));
    assert.equal(reviewCron({ ...daily, ...bad }), null);
  }
});

test("preview triage commits notes and lead requests together with independent project snapshots", () => {
  const state = intake();
  state.scratchpad.notes.push(archiveNote(createNote("NOTE-C", "Keep for later", NOW)));
  const settings = structuredClone(project);
  const inputs = structuredClone(context);
  const result = previewTriage(state, settings, inputs, { now: NOW + 10_000 });
  assert.equal(result.created, 2);
  assert.equal(result.requests.length, 2);
  assert.equal(result.scratchpad.notes.length, 3);
  assert.equal(result.scratchpad.notes[2].status, "archived");
  assert.deepEqual(result.requests[0].origin, { type: "scratchpad", noteId: "NOTE-A", revision: 1 });
  assert.equal(result.requests[0].text, "Inspect the queue");
  assert.equal(result.requests[0].policy, "lead");
  assert.equal(result.requests[0].runId, undefined);
  assert.equal(result.scratchpad.notes[0].requestId, result.requests[0].id);
  assert.equal(result.requests[0].settings.capturedAt, NOW + 10_000);
  settings.permissions.commands = "allow";
  inputs.contextFiles.push("secret.txt");
  assert.equal(result.requests[0].settings.permissions.commands, "deny");
  assert.deepEqual(result.requests[0].context.contextFiles, ["AGENTS.md"]);
  assert.equal(state.requests.length, 0);
  assert.equal(state.scratchpad.notes[0].status, "inbox");
});

test("repeated triage and reload never reissue sent or cancelled generated requests", () => {
  const first = previewTriage(intake(), project, context, { now: NOW });
  first.requests[0] = cancelRequest(first.requests[0], NOW + 1_000);
  const reloaded = { requests: JSON.parse(JSON.stringify(first.requests)), scratchpad: restoreScratchpad(JSON.parse(JSON.stringify(first.scratchpad))) };
  const again = previewTriage(reloaded, project, context, { now: NOW + 2_000 });
  assert.equal(again.created, 0);
  assert.equal(again.requests.length, 2);
  assert.equal(again.requests[0].status, "cancelled");
  const recovered = { ...reloaded, scratchpad: { ...reloaded.scratchpad, notes: reloaded.scratchpad.notes.map((n) => ({ ...n, status: "inbox", requestId: undefined })) } };
  const deduped = previewTriage(recovered, project, context, { now: NOW + 3_000 });
  assert.equal(deduped.requests.length, 2);
  assert.equal(deduped.scratchpad.notes[0].requestId, first.requests[0].id);
});

test("open edits or invalid project choices prevent a partial promotion and retain drafts", () => {
  const state = intake();
  state.scratchpad.edits["NOTE-A"] = "Unsubmitted updated note";
  const editing = previewTriage(state, project, context, { now: NOW });
  assert.match(editing.error, /Save or cancel/);
  assert.equal(editing.requests.length, 0);
  assert.equal(editing.scratchpad.edits["NOTE-A"], "Unsubmitted updated note");
  assert.equal(editing.scratchpad.notes[0].status, "inbox");
  const invalid = previewTriage(intake(), { ...project, model: "gemini-3.1" }, context, { now: NOW });
  assert.match(invalid.error, /not connected/);
  assert.equal(invalid.requests.length, 0);
  assert.equal(invalid.scratchpad.notes.every((n) => n.status === "inbox"), true);
});

test("malformed saved notes and configuration restore safely while valid drafts survive", () => {
  const note = createNote("NOTE-A", "Keep this note", NOW);
  const value = restoreScratchpad({ notes: [null, note, note, { ...note, id: "NOTE-B", createdAt: 1e100 }, { ...note, id: "NOTE-C", revision: 0 }, { ...note, id: "NOTE-D", status: "sent", requestId: 8 }], draft: "New unfinished thought", edits: { "NOTE-A": "An unfinished edit", "NOTE-NOPE": "orphan" }, config: { cadence: "wild" } });
  assert.deepEqual(value.notes, [note]);
  assert.equal(value.draft, "New unfinished thought");
  assert.deepEqual(value.edits, { "NOTE-A": "An unfinished edit" });
  assert.equal(value.config.time, "09:00");
  value.notes[0].text = "Changed restored copy";
  assert.equal(note.text, "Keep this note");
});
