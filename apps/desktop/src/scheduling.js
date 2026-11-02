import { validateCombo } from "./data.js";
import { validCapturedPermissions } from "./permissions.js";

export const POLICIES = [
  { id: "lead", label: "Lead decides" },
  { id: "next", label: "Next available" },
  { id: "at", label: "Schedule at…" },
];

export const localZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;

export function validateTiming(policy, value, now = Date.now()) {
  if (!POLICIES.some((p) => p.id === policy)) return { error: "Choose when the lead should schedule this request." };
  if (policy !== "at") return { scheduledAt: null, timeZone: localZone(), error: null };
  if (!value) return { error: "Choose a future date and time." };
  const parts = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(value);
  if (!parts) return { error: "Enter a valid local date and time." };
  const [year, month, day, hour, minute] = parts.slice(1).map(Number);
  const date = new Date(year, month - 1, day, hour, minute);
  if (year < 100) date.setFullYear(year);
  // Date normalizes invalid dates and DST gaps; reject rather than move the user's time.
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day || date.getHours() !== hour || date.getMinutes() !== minute)
    return { error: "That local time does not exist. Choose another date or time." };
  if (!Number.isFinite(date.getTime()) || date.getTime() <= now) return { error: "Choose a time in the future." };
  return { scheduledAt: date.getTime(), timeZone: localZone(), error: null };
}

export function formatRequestTime(at, timeZone, full = false) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short", day: "numeric", ...(full ? { year: "numeric" } : {}),
    hour: "numeric", minute: "2-digit", timeZone, timeZoneName: "shortOffset",
  }).format(at);
}

export const policyLabel = (request) => request.policy === "at"
  ? formatRequestTime(request.scheduledAt, request.timeZone, true)
  : POLICIES.find((p) => p.id === request.policy)?.label;

export function requestState(request, now = Date.now()) {
  if (request.status === "cancelled") return { id: "cancelled", label: "Cancelled" };
  if (request.policy === "at") return request.scheduledAt <= now
    ? { id: "due", label: "Due · awaiting lead" }
    : { id: "scheduled", label: "Scheduled" };
  return { id: "waiting", label: request.policy === "lead" ? "Awaiting plan" : "Queued request" };
}

export function createRequest({ id, text, policy, localTime, settings, context, origin, now = Date.now() }) {
  const timing = validateTiming(policy, localTime, now);
  const error = !text.trim() ? "Describe the task for the lead." : validateCombo(settings.model, settings.effort) || (!validCapturedPermissions(settings.permissions) && "Choose valid requested permissions.") || timing.error;
  if (error) return { error };
  return {
    error: null,
    request: {
      id, text: text.trim(), createdAt: now, updatedAt: now, policy,
      scheduledAt: timing.scheduledAt, timeZone: timing.timeZone,
      status: policy === "at" ? "scheduled" : "pending",
      settings: structuredClone({ ...settings, capturedAt: now }), context: structuredClone(context),
      ...(origin ? { origin: structuredClone(origin) } : {}),
    },
  };
}

export function rescheduleRequest(request, policy, localTime, now = Date.now()) {
  if (request.status === "cancelled") return { error: "Cancelled requests cannot be rescheduled." };
  const timing = validateTiming(policy, localTime, now);
  if (timing.error) return timing;
  return { error: null, request: { ...request, policy, scheduledAt: timing.scheduledAt, timeZone: timing.timeZone, status: policy === "at" ? "scheduled" : "pending", updatedAt: now } };
}

export function cancelRequest(request, now = Date.now()) {
  return request.status === "cancelled" ? request : { ...request, status: "cancelled", cancelledAt: now, updatedAt: now };
}

export function restoreRequests(value) {
  if (!Array.isArray(value)) return [];
  const ids = new Set();
  return value.filter((r) => {
    if (!r || typeof r !== "object" || typeof r.id !== "string" || !r.id.startsWith("REQ-") || ids.has(r.id) || typeof r.text !== "string" || !r.text.trim()) return false;
    if (!Number.isFinite(r.createdAt) || !Number.isFinite(new Date(r.createdAt).getTime()) || !["pending", "scheduled", "cancelled"].includes(r.status) || !POLICIES.some((p) => p.id === r.policy)) return false;
    if (r.policy === "at" && (!Number.isFinite(r.scheduledAt) || !Number.isFinite(new Date(r.scheduledAt).getTime()))) return false;
    try { new Intl.DateTimeFormat(undefined, { timeZone: r.timeZone }); } catch { return false; }
    if (typeof r.timeZone !== "string" || !r.settings || typeof r.settings.model !== "string" || typeof r.settings.effort !== "string" || typeof r.settings.mode !== "string" || !r.context || !Array.isArray(r.context.contextFiles) || !r.context.contextFiles.every((p) => typeof p === "string")) return false;
    if (["roots", "excludedFiles", "attached"].some((key) => r.context[key] != null && (!Array.isArray(r.context[key]) || !r.context[key].every((p) => typeof p === "string")))) return false;
    if (r.context.ignore != null && typeof r.context.ignore !== "string") return false;
    if (r.settings.access != null && typeof r.settings.access !== "string") return false;
    if (!validCapturedPermissions(r.settings.permissions)) return false;
    if (r.origin != null && (r.origin.type !== "scratchpad" || typeof r.origin.noteId !== "string" || !r.origin.noteId.startsWith("NOTE-") || !Number.isInteger(r.origin.revision) || r.origin.revision < 1)) return false;
    if (["budget", "concurrency"].some((key) => r.settings[key] != null && !Number.isFinite(r.settings[key]))) return false;
    if (r.settings.capturedAt != null && (!Number.isFinite(r.settings.capturedAt) || !Number.isFinite(new Date(r.settings.capturedAt).getTime()))) return false;
    ids.add(r.id);
    return true;
  }).map((r) => structuredClone(r));
}

export function localInputValue(at) {
  const date = new Date(at);
  const n = (v) => String(v).padStart(2, "0");
  return `${date.getFullYear()}-${n(date.getMonth() + 1)}-${n(date.getDate())}T${n(date.getHours())}:${n(date.getMinutes())}`;
}
