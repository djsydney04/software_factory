import { ACCESS } from "./data.js";
import { PERMISSION_FIELDS, PERMISSION_VALUES, permissionLabel } from "./permissions.js";

const DESCRIPTIONS = {
  commands: "Run project commands and tests.",
  network: "Make outbound network requests.",
  integrations: "Use connected services.",
};

export function PermissionFields({ value, onChange, ceiling, describe = false }) {
  return <div className="permission-fields">
    <label><span>Agent edits</span><select aria-label="Agent edits" value={value.access} onChange={(e) => onChange({ ...value, access: e.target.value })}>{ACCESS.map((a) => <option key={a.id} value={a.id} disabled={a.id === "worktree" && ceiling?.access === "read-only"}>{a.label}</option>)}</select>{describe && <small>Read only, or edit in an isolated worktree.</small>}</label>
    {PERMISSION_FIELDS.map(({ id, label }) => <label key={id}><span>{label}</span><select aria-label={label} value={value.permissions[id]} onChange={(e) => onChange({ ...value, permissions: { ...value.permissions, [id]: e.target.value } })}>{PERMISSION_VALUES.map((p) => <option key={p} value={p} disabled={ceiling && PERMISSION_VALUES.indexOf(p) > PERMISSION_VALUES.indexOf(ceiling.permissions[id])}>{permissionLabel(p)}</option>)}</select>{describe && <small>{DESCRIPTIONS[id]}</small>}</label>)}
  </div>;
}

export function PermissionFacts({ permissions }) {
  return PERMISSION_FIELDS.map(({ id, label }) => <div className="permission-fact" key={id}><dt>{label}</dt><dd>{permissionLabel(permissions?.[id])}</dd></div>);
}
