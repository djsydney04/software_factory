export const repositoryProvider = (provider) => provider === "github" ? "GitHub" : provider === "gitlab" ? "GitLab" : "Repository";
export const repositoryConnectionKey = (connection) => connection ? [connection.projectId, connection.provider, connection.host, connection.path, connection.connectedAt].join("|") : "disconnected";

// Provider text is untrusted; only the native allowlisted opener can leave the app.
export function safeRepositoryUrl(value, connection) {
  if (typeof value !== "string" || !connection || !["github.com", "gitlab.com"].includes(connection.host)) return null;
  try {
    const url = new URL(value);
    if (url.protocol !== "https:" || url.hostname !== connection.host || url.port || url.username || url.password || url.search || url.hash) return null;
    const root = `/${connection.path}`;
    const path = connection.provider === "github" ? url.pathname.toLowerCase() : url.pathname;
    const prefix = connection.provider === "github" ? root.toLowerCase() : root;
    return path === prefix || path.startsWith(`${prefix}/`) ? url.href : null;
  } catch { return null; }
}

export const initialRepositoryRead = () => ({ requestId: null, scope: null, status: "idle", value: null, error: "" });
export function repositoryReadReducer(state, action) {
  if (action.type === "start") return { requestId: action.requestId, scope: action.scope, status: "loading", value: null, error: "" };
  if (action.type === "clear") return initialRepositoryRead();
  if (state.requestId !== action.requestId || state.scope !== action.scope) return state;
  if (action.type === "resolved") return { ...state, status: "ready", value: action.value, error: "" };
  if (action.type === "rejected") return { ...state, status: "error", value: null, error: action.error };
  return state;
}

const CI_LABELS = { success: "Passed", failure: "Failed", running: "Running", pending: "Pending", cancelled: "Cancelled", skipped: "Skipped", unknown: "Unknown", none: "No checks" };
export function repositoryCiEvidence(detail) {
  const ci = detail?.ci;
  const items = Array.isArray(ci?.items) ? ci.items : [];
  const revision = typeof detail?.change?.headSha === "string" ? detail.change.headSha : "";
  const current = !!revision && ci?.revision === revision;
  const mismatched = items.some((item) => item.revision && item.revision !== revision);
  const complete = ci?.complete === true;
  const known = Object.hasOwn(CI_LABELS, ci?.state);
  const passed = current && complete && !mismatched && items.length > 0 && ci.state === "success" && items.every((item) => item.state === "success");
  return { items, current, complete, passed, revision,
    state: passed ? "success" : !current || mismatched || !complete || !known || ci?.state === "success" ? "unknown" : ci.state,
    label: !current || mismatched ? "Revision mismatch" : !complete ? "Partial CI results" : !items.length && ["success", "none", "unknown"].includes(ci?.state) ? "No checks reported" : passed ? "Passed" : ci?.state === "success" ? "Unverified CI results" : CI_LABELS[ci?.state] ?? "Unknown",
    limitations: Array.isArray(ci?.limitations) ? ci.limitations : [] };
}

export const repositoryCiLabel = (state) => CI_LABELS[state] ?? "Unknown";
export function repositoryFetchedLabel(value) {
  const time = Date.parse(value);
  return Number.isFinite(time) ? new Date(time).toLocaleString([], { dateStyle: "medium", timeStyle: "short" }) : "Time unavailable";
}
