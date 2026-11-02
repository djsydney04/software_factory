import { createRequest, localZone } from "./scheduling.js";

export const CADENCES = [
  { id: "daily", label: "Every morning" },
  { id: "hourly", label: "Every hour" },
  { id: "quarter-hour", label: "Every 15 minutes" },
];
const validDate = (value) => Number.isFinite(value) && Number.isFinite(new Date(value).getTime());
export const defaultReviewConfig = () => ({ enabled: true, cadence: "daily", time: "09:00", timeZone: localZone() });

export function validateReviewConfig(config) {
  if (!config || typeof config.enabled !== "boolean" || !CADENCES.some((c) => c.id === config.cadence)) return "Choose a review cadence.";
  if (config.cadence === "daily" && !/^([01]\d|2[0-3]):[0-5]\d$/.test(config.time)) return "Choose a valid review time.";
  if (typeof config.timeZone !== "string" || !config.timeZone.trim()) return "Enter an IANA time zone, such as America/Los_Angeles.";
  try {
    const zone = new Intl.DateTimeFormat(undefined, { timeZone: config.timeZone }).resolvedOptions().timeZone;
    if (/^[+-]/.test(zone)) return "Enter an IANA time zone, rather than a fixed offset.";
  } catch { return "Enter a valid IANA time zone."; }
  return null;
}

export function reviewCron(config) {
  if (validateReviewConfig(config)) return null;
  if (config.cadence === "hourly") return "0 * * * *";
  if (config.cadence === "quarter-hour") return "*/15 * * * *";
  const [hour, minute] = config.time.split(":").map(Number);
  return `${minute} ${hour} * * *`;
}

export function reviewLabel(config) {
  const cadence = config.cadence === "daily" ? `Every day at ${config.time}` : config.cadence === "hourly" ? "Every hour" : "Every 15 minutes";
  return `${cadence} · ${config.timeZone}${config.enabled ? "" : " · paused"}`;
}

export function createNote(id, text, now = Date.now()) {
  if (typeof text !== "string" || !text.trim()) return null;
  return { id, text: text.trim(), revision: 1, status: "inbox", createdAt: now, updatedAt: now };
}

export function editNote(note, text, now = Date.now()) {
  if (note.status !== "inbox" || !text.trim() || text.trim() === note.text) return note;
  return { ...note, text: text.trim(), revision: note.revision + 1, updatedAt: now };
}

export function archiveNote(note, now = Date.now()) {
  return note.status === "archived" ? note : { ...note, status: "archived", updatedAt: now };
}

export function unarchiveNote(note, now = Date.now()) {
  return note.status !== "archived" ? note : { ...note, status: note.requestId ? "sent" : "inbox", updatedAt: now };
}

function restoreDocument(document, notes, edits, draft) {
  if (typeof document?.text !== "string") {
    let text = "";
    const anchors = [];
    const ordered = notes.filter((note) => note.status !== "archived").sort((a, b) => a.createdAt - b.createdAt);
    for (const [index, note] of ordered.entries()) {
      if (index) text += "\n\n";
      const content = note.status === "inbox" && Object.hasOwn(edits, note.id) ? edits[note.id] : note.text;
      const start = text.length;
      text += content;
      if (content === note.text) anchors.push({ noteId: note.id, revision: note.revision, start, end: text.length });
    }
    if (draft) text += `${ordered.length ? "\n\n" : ""}${draft}`;
    return { text, anchors };
  }
  const { text } = document;
  const candidates = (Array.isArray(document.anchors) ? document.anchors : []).filter((anchor) => {
    if (!anchor || !Number.isInteger(anchor.start) || !Number.isInteger(anchor.end) || anchor.start < 0 || anchor.end <= anchor.start || anchor.end > text.length) return false;
    const note = notes.find((item) => item.id === anchor.noteId && item.revision === anchor.revision);
    return note && text.slice(anchor.start, anchor.end) === note.text;
  }).map(({ noteId, revision, start, end }) => ({ noteId, revision, start, end })).sort((a, b) => a.start - b.start);
  const counts = new Map();
  for (const anchor of candidates) counts.set(anchor.noteId, (counts.get(anchor.noteId) || 0) + 1);
  const anchors = [];
  for (let index = 0; index < candidates.length;) {
    let next = index + 1;
    let end = candidates[index].end;
    while (next < candidates.length && candidates[next].start < end) end = Math.max(end, candidates[next++].end);
    if (next === index + 1 && counts.get(candidates[index].noteId) === 1) anchors.push(candidates[index]);
    index = next;
  }
  return { text, anchors };
}

export function restoreScratchpad(value) {
  const ids = new Set();
  const notes = (Array.isArray(value?.notes) ? value.notes : []).filter((n) => {
    if (!n || typeof n.id !== "string" || !n.id.startsWith("NOTE-") || ids.has(n.id) || typeof n.text !== "string" || !n.text.trim() || !Number.isInteger(n.revision) || n.revision < 1 || !["inbox", "sent", "archived"].includes(n.status) || !validDate(n.createdAt) || !validDate(n.updatedAt)) return false;
    if ((n.status === "sent" || n.requestId != null) && (typeof n.requestId !== "string" || !n.requestId.startsWith("REQ-"))) return false;
    ids.add(n.id);
    return true;
  }).map((n) => ({ ...structuredClone(n), status: n.requestId && n.status !== "archived" ? "sent" : n.status }));
  const draft = typeof value?.draft === "string" ? value.draft : "";
  const edits = Object.fromEntries(Object.entries(value?.edits && typeof value.edits === "object" ? value.edits : {}).filter(([id, text]) => notes.some((n) => n.id === id && n.status === "inbox") && typeof text === "string"));
  return {
    notes, draft, edits, document: restoreDocument(value?.document, notes, edits, draft),
    config: !validateReviewConfig(value?.config) ? structuredClone(value.config) : defaultReviewConfig(),
    lastPreviewAt: validDate(value?.lastPreviewAt) ? value.lastPreviewAt : null,
  };
}

export function updateScratchpadDocument(value, text, change) {
  if (typeof text !== "string") return value;
  const scratchpad = restoreScratchpad(value);
  const before = scratchpad.document.text;
  if (before === text) return scratchpad;
  const replacementEnd = text.length - (before.length - change?.end);
  const precise = change?.before === before && Number.isInteger(change.start) && Number.isInteger(change.end) && change.start >= 0 && change.end >= change.start && change.end <= before.length && replacementEnd >= change.start && text.slice(0, change.start) === before.slice(0, change.start) && text.slice(replacementEnd) === before.slice(change.end);
  let start = change?.start;
  let oldEnd = change?.end;
  if (!precise) {
    let prefix = 0;
    let suffix = 0;
    while (prefix < before.length && prefix < text.length && before[prefix] === text[prefix]) prefix++;
    while (suffix < before.length && suffix < text.length && before[before.length - suffix - 1] === text[text.length - suffix - 1]) suffix++;
    // Repeated text can put the same edit in several places; discard anchors across every possible placement.
    const overlap = Math.max(0, prefix + suffix - Math.min(before.length, text.length));
    start = prefix - overlap;
    oldEnd = before.length - suffix + overlap;
  }
  const delta = text.length - before.length;
  // Keep only unchanged boundary ranges; repeated text elsewhere cannot recover an edited anchor.
  const anchors = scratchpad.document.anchors.flatMap((anchor) => anchor.end <= start ? [anchor] : anchor.start >= oldEnd ? [{ ...anchor, start: anchor.start + delta, end: anchor.end + delta }] : []);
  return { ...scratchpad, document: { text, anchors } };
}

export function applyDocumentEdit(value, text, { history = { past: [], future: [] }, change, inputType } = {}) {
  const scratchpad = restoreScratchpad(value);
  if (typeof text !== "string" || text === scratchpad.document.text) return { value, history };
  const past = (Array.isArray(history?.past) ? history.past : []).filter((frame) => typeof frame?.text === "string");
  const future = (Array.isArray(history?.future) ? history.future : []).filter((frame) => typeof frame?.text === "string");
  const undo = inputType === "historyUndo";
  const redo = inputType === "historyRedo";
  const frames = undo ? past : future;
  if (undo || redo) {
    const index = frames.findLastIndex((frame) => frame.text === text);
    if (index >= 0) {
      const document = restoreScratchpad({ ...scratchpad, document: frames[index] }).document;
      const skipped = frames.slice(index + 1).reverse();
      return {
        value: { ...scratchpad, document },
        history: undo
          ? { past: past.slice(0, index), future: [...future, scratchpad.document, ...skipped] }
          : { past: [...past, scratchpad.document, ...skipped], future: future.slice(0, index) },
      };
    }
  }
  return {
    value: updateScratchpadDocument(scratchpad, text, undo || redo ? undefined : change),
    history: { past: [...past, scratchpad.document], future: [] },
  };
}

function nonblankRange(text, start, end) {
  while (start < end && /\s/.test(text[start])) start++;
  while (end > start && /\s/.test(text[end - 1])) end--;
  return start >= end ? null : { start, end, text: text.slice(start, end) };
}

export function pendingDocumentRanges(value) {
  const { document } = restoreScratchpad(value);
  const ranges = [];
  const separator = /\r?\n[\t ]*\r?\n/g;
  let start = 0;
  const paragraphs = [];
  for (const match of document.text.matchAll(separator)) { paragraphs.push([start, match.index]); start = match.index + match[0].length; }
  paragraphs.push([start, document.text.length]);
  for (const [paragraphStart, paragraphEnd] of paragraphs) {
    let cursor = paragraphStart;
    for (const anchor of document.anchors) {
      if (anchor.end <= cursor || anchor.start >= paragraphEnd) continue;
      const range = nonblankRange(document.text, cursor, Math.max(cursor, anchor.start));
      if (range) ranges.push(range);
      cursor = Math.max(cursor, anchor.end);
    }
    const range = nonblankRange(document.text, cursor, paragraphEnd);
    if (range) ranges.push(range);
  }
  return ranges;
}

export function previewDocumentTriage(intake, settings, context, { now = Date.now(), makeNoteId = () => `NOTE-${crypto.randomUUID().replaceAll("-", "").slice(0, 8).toUpperCase()}` } = {}) {
  const scratchpad = restoreScratchpad(intake.scratchpad);
  const notes = [...scratchpad.notes];
  const anchors = [...scratchpad.document.anchors];
  for (const [index, range] of pendingDocumentRanges(scratchpad).entries()) {
    const id = makeNoteId(range, index);
    if (typeof id !== "string" || !id.startsWith("NOTE-") || notes.some((note) => note.id === id)) return { ...intake, created: 0, error: "Note identity collision. Try preview triage again." };
    const note = createNote(id, range.text, now);
    notes.push(note);
    anchors.push({ noteId: id, revision: note.revision, start: range.start, end: range.end });
  }
  const anchored = new Set(anchors.map((anchor) => anchor.noteId));
  const eligible = notes.filter((note) => anchored.has(note.id));
  const result = previewTriage({ requests: intake.requests, scratchpad: { ...scratchpad, notes: eligible, edits: {} } }, settings, context, { now });
  if (result.error) return { ...intake, created: 0, error: result.error };
  const updated = new Map(result.scratchpad.notes.map((note) => [note.id, note]));
  return { ...intake, requests: result.requests, scratchpad: { ...scratchpad, notes: notes.map((note) => updated.get(note.id) || note), document: { text: scratchpad.document.text, anchors: anchors.sort((a, b) => a.start - b.start) }, lastPreviewAt: now }, created: result.created, error: null };
}

export function completedDocumentRanges(value, requests, runs) {
  const scratchpad = restoreScratchpad(value);
  return scratchpad.document.anchors.flatMap((anchor) => {
    const note = scratchpad.notes.find((item) => item.id === anchor.noteId);
    const request = requests.find((item) => item.id === note.requestId && item.origin?.type === "scratchpad" && item.origin.noteId === anchor.noteId && item.origin.revision === anchor.revision);
    if (!request || request.status === "cancelled" || request.text !== note.text) return [];
    const linked = runs.filter((run) => run.requestId === request.id).sort((a, b) => (b.attempt || 0) - (a.attempt || 0) || (b.startedAt || 0) - (a.startedAt || 0));
    const latest = linked[0];
    return latest?.status === "succeeded" ? [{ start: anchor.start, end: anchor.end, noteId: anchor.noteId, requestId: request.id, runId: latest.id }] : [];
  });
}

export function previewTriage(intake, settings, context, { now = Date.now(), makeId = (note) => `REQ-SP-${note.id.slice(5)}-${note.revision}` } = {}) {
  if (intake.scratchpad.notes.some((n) => n.status === "inbox" && Object.hasOwn(intake.scratchpad.edits, n.id)))
    return { ...intake, created: 0, error: "Save or cancel open note edits before preview triage." };
  const requests = [...intake.requests];
  let created = 0;
  const notes = [];
  for (const note of intake.scratchpad.notes) {
    if (note.status !== "inbox") { notes.push(note); continue; }
    const existing = requests.find((r) => r.id === note.requestId || (r.origin?.type === "scratchpad" && r.origin.noteId === note.id && r.origin.revision === note.revision));
    if (existing) { notes.push({ ...note, status: "sent", requestId: existing.id }); continue; }
    const id = makeId(note);
    if (requests.some((r) => r.id === id)) return { ...intake, created: 0, error: "Request identity collision. Try preview triage again." };
    const result = createRequest({ id, text: note.text, policy: "lead", localTime: "", settings, context, now, origin: { type: "scratchpad", noteId: note.id, revision: note.revision } });
    if (result.error) return { ...intake, created: 0, error: result.error };
    requests.push(result.request);
    notes.push({ ...note, status: "sent", requestId: id, updatedAt: now });
    created++;
  }
  return { requests, scratchpad: { ...intake.scratchpad, notes, lastPreviewAt: now }, created, error: null };
}
