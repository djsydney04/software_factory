import assert from "node:assert/strict";
import test from "node:test";
import { applyDocumentEdit, completedDocumentRanges, createNote, pendingDocumentRanges, previewDocumentTriage, restoreScratchpad, updateScratchpadDocument } from "../src/scratchpad.js";
import { cancelRequest } from "../src/scheduling.js";

const NOW = 1_800_000_000_000;
const settings = { model: "codex-gpt-5.3", effort: "medium", mode: "ask", access: "read-only", permissions: { commands: "deny", network: "ask", integrations: "deny" } };
const context = { contextFiles: ["AGENTS.md"], roots: ["src/"], attached: [] };
const document = (text) => updateScratchpadDocument(restoreScratchpad(null), text);
const triage = (text) => previewDocumentTriage({ requests: [], scratchpad: document(text) }, settings, context, { now: NOW, makeNoteId: (_range, index) => `NOTE-${index + 1}` });

test("legacy notes migrate once into a document without losing drafts, edits, history, or configuration", () => {
  const older = { ...createNote("NOTE-OLD", "Already submitted", NOW), status: "sent", requestId: "REQ-OLD" };
  const newer = createNote("NOTE-EDIT", "Original note", NOW + 1);
  const archived = { ...createNote("NOTE-ARCHIVED", "Archived history", NOW + 2), status: "archived" };
  const config = { enabled: false, cadence: "hourly", time: "09:00", timeZone: "UTC" };
  const value = restoreScratchpad({ notes: [newer, archived, older], edits: { "NOTE-EDIT": "  Unsaved replacement\nwith a second line  " }, draft: "  Trailing unfinished thought\n", config, lastPreviewAt: NOW });
  assert.equal(value.document.text, "Already submitted\n\n  Unsaved replacement\nwith a second line  \n\n  Trailing unfinished thought\n");
  assert.deepEqual(value.document.anchors, [{ noteId: "NOTE-OLD", revision: 1, start: 0, end: 17 }]);
  assert.deepEqual(value.notes, [newer, archived, older]);
  assert.equal(value.edits["NOTE-EDIT"], "  Unsaved replacement\nwith a second line  ");
  assert.equal(value.draft, "  Trailing unfinished thought\n");
  assert.deepEqual(value.config, config);
  assert.deepEqual(restoreScratchpad(JSON.parse(JSON.stringify(value))), value);
  const cleared = updateScratchpadDocument(value, "");
  assert.equal(restoreScratchpad(cleared).document.text, "");
  assert.equal(cleared.notes.length, 3);
  assert.equal(cleared.draft, value.draft);
});

test("restoration preserves canonical text and rejects stale, overlapping, duplicate, or malformed anchors", () => {
  const note = createNote("NOTE-A", "Repeat", NOW);
  const other = createNote("NOTE-B", "Repeat", NOW);
  const base = { notes: [note, other], draft: "Do not append this legacy draft", document: { text: "Repeat\n\nRepeat", anchors: [{ noteId: "NOTE-A", revision: 1, start: 0, end: 6 }, { noteId: "NOTE-B", revision: 1, start: 8, end: 14 }] } };
  assert.equal(restoreScratchpad(base).document.anchors.length, 2);
  for (const bad of [null, { noteId: "NOTE-MISSING", revision: 1, start: 0, end: 6 }, { noteId: "NOTE-A", revision: 2, start: 0, end: 6 }, { noteId: "NOTE-A", revision: 1, start: -1, end: 6 }, { noteId: "NOTE-A", revision: 1, start: 1, end: 7 }, { noteId: "NOTE-A", revision: 1, start: 0, end: Infinity }]) {
    const result = restoreScratchpad({ ...base, document: { ...base.document, anchors: [bad] } });
    assert.equal(result.document.text, "Repeat\n\nRepeat");
    assert.deepEqual(result.document.anchors, []);
  }
  const overlap = restoreScratchpad({ ...base, document: { ...base.document, anchors: [{ noteId: "NOTE-A", revision: 1, start: 0, end: 6 }, { noteId: "NOTE-B", revision: 1, start: 0, end: 6 }] } });
  assert.deepEqual(overlap.document.anchors, []);
  const duplicateIdentity = restoreScratchpad({ ...base, document: { ...base.document, anchors: [{ noteId: "NOTE-A", revision: 1, start: 0, end: 6 }, { noteId: "NOTE-A", revision: 1, start: 8, end: 14 }] } });
  assert.deepEqual(duplicateIdentity.document.anchors, []);
});

test("document edits shift untouched ranges and invalidate edited source without finding repeated text elsewhere", () => {
  const first = triage("Repeat\n\nRepeat").scratchpad;
  const prefixed = updateScratchpadDocument(first, "Prefix\n\nRepeat\n\nRepeat");
  assert.deepEqual(prefixed.document.anchors.map(({ noteId, start, end }) => ({ noteId, start, end })), [{ noteId: "NOTE-1", start: 8, end: 14 }, { noteId: "NOTE-2", start: 16, end: 22 }]);
  const edited = updateScratchpadDocument(prefixed, "Prefix\n\nChanged\n\nRepeat");
  assert.deepEqual(edited.document.anchors.map(({ noteId, start, end }) => ({ noteId, start, end })), [{ noteId: "NOTE-2", start: 17, end: 23 }]);
  assert.deepEqual(pendingDocumentRanges(edited).map((range) => range.text), ["Prefix", "Changed"]);
  const removed = updateScratchpadDocument(edited, "Prefix\n\nChanged");
  assert.deepEqual(removed.document.anchors, []);
  assert.equal(first.notes[0].text, "Repeat");
  assert.equal(first.notes[0].requestId, "REQ-SP-1-1");
});

test("ambiguous repeated-text deletion or insertion cannot transfer completion to another occurrence", () => {
  const state = triage("Repeat\n\nRepeat");
  const succeeded = { id: "R-DONE", requestId: state.requests[0].id, attempt: 1, startedAt: NOW, status: "succeeded" };
  assert.equal(completedDocumentRanges(state.scratchpad, state.requests, [succeeded]).length, 1);
  const deletedFirst = updateScratchpadDocument(state.scratchpad, "Repeat");
  assert.deepEqual(deletedFirst.document.anchors, []);
  assert.deepEqual(completedDocumentRanges(deletedFirst, state.requests, [succeeded]), []);
  assert.deepEqual(pendingDocumentRanges(deletedFirst), [{ start: 0, end: 6, text: "Repeat" }]);
  const single = triage("Repeat");
  const duplicated = updateScratchpadDocument(single.scratchpad, "Repeat\n\nRepeat");
  assert.deepEqual(duplicated.document.anchors, []);
  const unchanged = updateScratchpadDocument(state.scratchpad, state.scratchpad.document.text);
  assert.deepEqual(unchanged.document.anchors, state.scratchpad.document.anchors);
  const surrounded = triage("Unique before\n\nRepeat\n\nRepeat\n\nUnique after");
  const reduced = updateScratchpadDocument(surrounded.scratchpad, "Unique before\n\nRepeat\n\nUnique after");
  assert.deepEqual(reduced.document.anchors.map(({ noteId, start, end }) => ({ noteId, start, end })), [{ noteId: "NOTE-1", start: 0, end: 13 }, { noteId: "NOTE-4", start: 23, end: 35 }]);
});

test("verified native edit intervals distinguish identical occurrences and retain normal appended paragraphs", () => {
  const state = triage("Repeat\n\nRepeat");
  const success = { id: "R-DONE", requestId: state.requests[0].id, attempt: 1, startedAt: NOW, status: "succeeded" };
  const before = state.scratchpad.document.text;
  const deletedFirst = updateScratchpadDocument(state.scratchpad, "Repeat", { start: 0, end: 8, before });
  assert.deepEqual(deletedFirst.document.anchors, [{ noteId: "NOTE-2", revision: 1, start: 0, end: 6 }]);
  assert.deepEqual(completedDocumentRanges(deletedFirst, state.requests, [success]), []);
  assert.deepEqual(pendingDocumentRanges(deletedFirst), []);
  const deletedSecond = updateScratchpadDocument(state.scratchpad, "Repeat", { start: 6, end: 14, before });
  assert.deepEqual(deletedSecond.document.anchors, [{ noteId: "NOTE-1", revision: 1, start: 0, end: 6 }]);
  assert.equal(completedDocumentRanges(deletedSecond, state.requests, [success]).length, 1);
  const sentence = triage("One task.").scratchpad;
  const appended = updateScratchpadDocument(sentence, "One task.\n\nAnother task.", { before: "One task.", start: 9, end: 9 });
  assert.deepEqual(appended.document.anchors, sentence.document.anchors);
  assert.deepEqual(pendingDocumentRanges(appended).map((range) => range.text), ["Another task."]);
  const insertedDuplicate = updateScratchpadDocument(sentence, "One task.\n\nOne task.", { before: "One task.", start: 0, end: 0 });
  assert.deepEqual(insertedDuplicate.document.anchors, [{ noteId: "NOTE-1", revision: 1, start: 11, end: 20 }]);
  for (const hint of [null, { before: "Stale text", start: 0, end: 8 }, { before, start: -1, end: 8 }, { before, start: 0.5, end: 8 }, { before, start: 8, end: 0 }, { before, start: 0, end: 100 }, { before, start: 0, end: 2 }]) {
    assert.deepEqual(updateScratchpadDocument(state.scratchpad, "Repeat", hint).document.anchors, []);
  }
  const mismatchedOutside = updateScratchpadDocument(sentence, "One task.\n\nAnother task.", { before: "One task.", start: 0, end: 0 });
  assert.deepEqual(mismatchedOutside.document.anchors, []);
});

test("native undo restores submitted anchors without rolling back current metadata or reissuing work", () => {
  const state = triage("Inspect the queue.");
  const original = state.scratchpad.document.text;
  const changed = applyDocumentEdit(state.scratchpad, "Rewrite the queue.", { change: { before: original, start: 0, end: original.length } });
  assert.deepEqual(changed.value.document.anchors, []);
  const current = { ...changed.value, config: { enabled: false, cadence: "hourly", time: "09:00", timeZone: "UTC" }, lastPreviewAt: NOW + 1, draft: "Legacy draft retained", notes: [...changed.value.notes, createNote("NOTE-LATER", "A later history record", NOW + 1)] };
  const undone = applyDocumentEdit(current, original, { history: changed.history, inputType: "historyUndo" });
  assert.deepEqual(undone.value.document.anchors, state.scratchpad.document.anchors);
  assert.deepEqual(undone.value.config, current.config);
  assert.deepEqual(undone.value.notes, current.notes);
  assert.equal(undone.value.lastPreviewAt, NOW + 1);
  assert.equal(undone.value.draft, "Legacy draft retained");
  const requestsBefore = structuredClone(state.requests);
  const repeated = previewDocumentTriage({ ...state, scratchpad: undone.value }, settings, context, { now: NOW + 2, makeNoteId: () => { throw new Error("Undo must not duplicate a submitted request"); } });
  assert.equal(repeated.created, 0);
  assert.deepEqual(repeated.requests, requestsBefore);
  assert.equal(repeated.requests[0].settings.permissions.commands, "deny");
  assert.deepEqual(Object.keys(changed.history.past[0]).sort(), ["anchors", "text"]);
});

test("native undo and redo preserve distinct source identity when identical paragraphs are deleted", () => {
  const state = triage("Repeat\n\nRepeat");
  const before = state.scratchpad.document.text;
  const run = { id: "R-DONE", requestId: state.requests[0].id, attempt: 1, startedAt: NOW, status: "succeeded" };
  const edited = applyDocumentEdit(state.scratchpad, "Repeat", { change: { before, start: 0, end: 8 } });
  assert.equal(edited.value.document.anchors[0].noteId, "NOTE-2");
  const undone = applyDocumentEdit(edited.value, before, { history: edited.history, inputType: "historyUndo" });
  assert.deepEqual(undone.value.document.anchors, state.scratchpad.document.anchors);
  assert.equal(completedDocumentRanges(undone.value, state.requests, [run])[0].noteId, "NOTE-1");
  const redone = applyDocumentEdit(undone.value, "Repeat", { history: undone.history, inputType: "historyRedo" });
  assert.deepEqual(redone.value.document.anchors, [{ noteId: "NOTE-2", revision: 1, start: 0, end: 6 }]);
  assert.deepEqual(completedDocumentRanges(redone.value, state.requests, [run]), []);
  assert.deepEqual(pendingDocumentRanges(redone.value), []);
});

test("browser-grouped undo and redo can cross several edit snapshots while retaining source anchors", () => {
  let edited = { value: triage("Base").scratchpad, history: { past: [], future: [] } };
  for (const text of ["Base one", "Base one two", "Base one two three"]) {
    const before = edited.value.document.text;
    edited = applyDocumentEdit(edited.value, text, { history: edited.history, change: { before, start: before.length, end: before.length } });
  }
  const undone = applyDocumentEdit(edited.value, "Base", { history: edited.history, inputType: "historyUndo" });
  assert.deepEqual(undone.value.document.anchors, [{ noteId: "NOTE-1", revision: 1, start: 0, end: 4 }]);
  const partialRedo = applyDocumentEdit(undone.value, "Base one two", { history: undone.history, inputType: "historyRedo" });
  assert.equal(partialRedo.value.document.text, "Base one two");
  const fullRedo = applyDocumentEdit(partialRedo.value, "Base one two three", { history: partialRedo.history, inputType: "historyRedo" });
  assert.deepEqual(fullRedo.value.document, edited.value.document);
  const groupedAgain = applyDocumentEdit(fullRedo.value, "Base one", { history: fullRedo.history, inputType: "historyUndo" });
  assert.equal(groupedAgain.value.document.text, "Base one");
  assert.deepEqual(groupedAgain.value.document.anchors, undone.value.document.anchors);
});

test("ordinary replacement and unrecognized native history never infer identity from matching old text", () => {
  const state = triage("Original task").scratchpad;
  const edited = applyDocumentEdit(state, "Changed task");
  const replacement = applyDocumentEdit(edited.value, "Original task", { history: edited.history, inputType: "insertFromPaste" });
  assert.deepEqual(replacement.value.document.anchors, []);
  assert.deepEqual(replacement.history.future, []);
  const same = applyDocumentEdit(replacement.value, "Original task", { history: replacement.history, inputType: "historyUndo" });
  assert.equal(same.value, replacement.value);
  assert.equal(same.history, replacement.history);
  const duplicate = triage("Repeat\n\nRepeat").scratchpad;
  const unknown = applyDocumentEdit(duplicate, "Repeat", { history: { past: [], future: [] }, inputType: "historyUndo", change: { before: duplicate.document.text, start: 6, end: 14 } });
  assert.deepEqual(unknown.value.document.anchors, []);
  const malformedFrame = { text: "Repeat", anchors: [{ noteId: "NOTE-1", revision: 99, start: 0, end: 6 }] };
  const restored = applyDocumentEdit(duplicate, "Repeat", { history: { past: [malformedFrame], future: [] }, inputType: "historyUndo" });
  assert.deepEqual(restored.value.document.anchors, []);
});

test("pending text consists of nonblank paragraphs minus submitted source ranges, including appended material", () => {
  const first = triage("First line\nsecond line\n\nOther paragraph").scratchpad;
  const appended = updateScratchpadDocument(first, "First line\nsecond line\n\nOther paragraph and an addition\n \n  A fresh paragraph  \n\n");
  const ranges = pendingDocumentRanges(appended);
  assert.deepEqual(ranges.map((range) => range.text), ["and an addition", "A fresh paragraph"]);
  for (const range of ranges) assert.equal(appended.document.text.slice(range.start, range.end), range.text);
  const note = createNote("NOTE-MULTI", "One\n\nTwo", NOW);
  const anchoredAcrossParagraphs = restoreScratchpad({ notes: [note], document: { text: note.text, anchors: [{ noteId: note.id, revision: 1, start: 0, end: 8 }] } });
  assert.deepEqual(pendingDocumentRanges(anchoredAcrossParagraphs), []);
  assert.deepEqual(pendingDocumentRanges(document(" \n\t\n ")), []);
});

test("document triage snapshots project choices atomically, retains cancelled provenance, and never issues removed inbox notes", () => {
  const removed = createNote("NOTE-LEGACY", "No longer in the document", NOW - 1);
  const scratchpad = { ...document("First task\n\nSecond task"), notes: [removed], edits: { [removed.id]: "An older open edit" } };
  const project = structuredClone(settings);
  const inputs = structuredClone(context);
  const first = previewDocumentTriage({ scratchpad, requests: [] }, project, inputs, { now: NOW, makeNoteId: (_range, index) => `NOTE-NEW-${index}` });
  assert.equal(first.error, null);
  assert.equal(first.created, 2);
  assert.deepEqual(first.requests.map((request) => request.text), ["First task", "Second task"]);
  assert.equal(first.scratchpad.notes[0].status, "inbox");
  assert.equal(first.scratchpad.edits[removed.id], "An older open edit");
  assert.equal(first.requests[0].policy, "lead");
  assert.equal(first.requests[0].runId, undefined);
  project.permissions.network = "allow";
  inputs.contextFiles.push("later.txt");
  assert.equal(first.requests[0].settings.permissions.network, "ask");
  assert.deepEqual(first.requests[0].context.contextFiles, ["AGENTS.md"]);
  first.requests[0] = cancelRequest(first.requests[0], NOW + 1);
  const restored = { scratchpad: restoreScratchpad(JSON.parse(JSON.stringify(first.scratchpad))), requests: JSON.parse(JSON.stringify(first.requests)) };
  const repeated = previewDocumentTriage(restored, settings, context, { now: NOW + 2, makeNoteId: () => { throw new Error("No new note should be made"); } });
  assert.equal(repeated.created, 0);
  assert.equal(repeated.requests.length, 2);
  assert.equal(repeated.requests[0].status, "cancelled");
  assert.deepEqual(pendingDocumentRanges(repeated.scratchpad), []);
  const appended = { ...restored, scratchpad: updateScratchpadDocument(restored.scratchpad, "First task\n\nSecond task\n\nInvestigate the third issue.") };
  const next = previewDocumentTriage(appended, settings, context, { now: NOW + 3, makeNoteId: () => "NOTE-THIRD" });
  assert.equal(next.created, 1);
  assert.equal(next.requests.length, 3);
  assert.equal(next.requests[2].text, "Investigate the third issue.");
});

test("invalid choices or note/request identity collisions leave the entire intake unchanged", () => {
  const intake = { requests: [], scratchpad: document("One\n\nTwo") };
  const snapshot = structuredClone(intake);
  const invalid = previewDocumentTriage(intake, { ...settings, model: "gemini-3.1" }, context, { now: NOW, makeNoteId: (_range, index) => `NOTE-${index}` });
  assert.match(invalid.error, /not connected/);
  assert.equal(invalid.created, 0);
  assert.equal(invalid.scratchpad, intake.scratchpad);
  const collision = previewDocumentTriage(intake, settings, context, { now: NOW, makeNoteId: () => "NOTE-SAME" });
  assert.match(collision.error, /collision/);
  assert.equal(collision.created, 0);
  assert.equal(collision.requests, intake.requests);
  const oldRequest = { id: "REQ-SP-0-1", origin: { type: "scratchpad", noteId: "NOTE-OTHER", revision: 1 } };
  const requestCollision = previewDocumentTriage({ ...intake, requests: [oldRequest] }, settings, context, { now: NOW, makeNoteId: (_range, index) => `NOTE-${index}` });
  assert.match(requestCollision.error, /collision/);
  assert.equal(requestCollision.created, 0);
  assert.equal(requestCollision.scratchpad, intake.scratchpad);
  assert.deepEqual(intake, snapshot);
});

test("completion highlights require exact immutable source identity and the latest linked run to succeed", () => {
  const state = triage("A completed task\n\nAnother task");
  const request = state.requests[0];
  const success = { id: "R-SUCCESS", requestId: request.id, attempt: 1, startedAt: NOW, status: "succeeded" };
  const expected = [{ start: 0, end: 16, noteId: "NOTE-1", requestId: request.id, runId: "R-SUCCESS" }];
  assert.deepEqual(completedDocumentRanges(state.scratchpad, state.requests, [success]), expected);
  assert.deepEqual(completedDocumentRanges(state.scratchpad, state.requests, []), []);
  for (const status of ["queued", "running", "needs_approval", "failed", "interrupted", "cancelled"]) {
    assert.deepEqual(completedDocumentRanges(state.scratchpad, state.requests, [success, { ...success, id: "R-LATER", attempt: 2, startedAt: NOW + 1, status }]), []);
  }
  assert.deepEqual(completedDocumentRanges(state.scratchpad, state.requests, [{ ...success, requestId: "REQ-UNRELATED" }]), []);
  for (const changed of [{ ...request, status: "cancelled" }, { ...request, text: "Another source" }, { ...request, origin: { ...request.origin, noteId: "NOTE-2" } }, { ...request, origin: { ...request.origin, revision: 2 } }, { ...request, origin: undefined }]) {
    assert.deepEqual(completedDocumentRanges(state.scratchpad, [changed], [success]), []);
  }
  const mismatchedLink = { ...state.scratchpad, notes: state.scratchpad.notes.map((note) => note.id === "NOTE-1" ? { ...note, requestId: "REQ-DIFFERENT" } : note) };
  assert.deepEqual(completedDocumentRanges(mismatchedLink, state.requests, [success]), []);
  const rewritten = updateScratchpadDocument(state.scratchpad, "A changed task\n\nAnother task");
  assert.deepEqual(completedDocumentRanges(rewritten, state.requests, [success]), []);
  const shifted = updateScratchpadDocument(state.scratchpad, "New note\n\nA completed task\n\nAnother task");
  assert.deepEqual(completedDocumentRanges(shifted, state.requests, [success]), [{ ...expected[0], start: 10, end: 26 }]);
});
