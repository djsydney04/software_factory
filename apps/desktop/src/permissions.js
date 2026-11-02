export const PERMISSION_FIELDS = [
  { id: "commands", label: "Commands" },
  { id: "network", label: "Network" },
  { id: "integrations", label: "Integrations" },
];
export const PERMISSION_VALUES = ["deny", "ask", "allow"];
export const DEFAULT_PERMISSIONS = { commands: "ask", network: "ask", integrations: "ask" };
export const permissionLabel = (value) => PERMISSION_VALUES.includes(value) ? ({ deny: "Deny", ask: "Ask", allow: "Allow" })[value] : "Not captured";

export function normalizePermissions(value) {
  return Object.fromEntries(PERMISSION_FIELDS.map(({ id }) => [id, PERMISSION_VALUES.includes(value?.[id]) ? value[id] : "ask"]));
}

export function validCapturedPermissions(value) {
  return value == null || (typeof value === "object" && !Array.isArray(value) && PERMISSION_FIELDS.every(({ id }) => PERMISSION_VALUES.includes(value[id])));
}

export function normalizeAccess(value) {
  return value === "worktree" ? "worktree" : "read-only";
}

export function resolvePermissions(project, override) {
  const permissions = normalizePermissions(project.permissions);
  if (override) {
    const requested = normalizePermissions(override.permissions);
    for (const { id } of PERMISSION_FIELDS)
      permissions[id] = PERMISSION_VALUES[Math.min(PERMISSION_VALUES.indexOf(permissions[id]), PERMISSION_VALUES.indexOf(requested[id]))];
  }
  return { access: project.access === "worktree" && (!override || override.access === "worktree") ? "worktree" : "read-only", permissions };
}
