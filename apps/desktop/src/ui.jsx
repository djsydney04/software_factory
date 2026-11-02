import { useEffect, useRef } from "react";
import { X } from "@phosphor-icons/react";
import { STATUS, MODELS, EFFORTS, MODES, model } from "./data.js";

export function Tabs({ id, label, items, value, onChange, vertical = false, className = "" }) {
  const refs = useRef({});
  const onKey = (e, i) => {
    let n = null;
    if (e.key === "ArrowRight" || e.key === "ArrowDown") n = (i + 1) % items.length;
    else if (e.key === "ArrowLeft" || e.key === "ArrowUp") n = (i - 1 + items.length) % items.length;
    else if (e.key === "Home") n = 0;
    else if (e.key === "End") n = items.length - 1;
    if (n === null) return;
    e.preventDefault();
    onChange(items[n].id);
    refs.current[items[n].id]?.focus();
  };
  return (
    <div role="tablist" aria-label={label} aria-orientation={vertical ? "vertical" : "horizontal"} className={`${vertical ? "vtabs" : "seg"} ${className}`}>
      {items.map((it, i) => (
        <button
          key={it.id}
          ref={(el) => (refs.current[it.id] = el)}
          type="button"
          role="tab"
          id={`${id}-tab-${it.id}`}
          aria-selected={value === it.id}
          aria-controls={`${id}-panel`}
          tabIndex={value === it.id ? 0 : -1}
          onClick={() => onChange(it.id)}
          onKeyDown={(e) => onKey(e, i)}
        >
          {it.icon}
          <span>{it.label}</span>
          {it.count != null && <span className="count">{it.count}</span>}
        </button>
      ))}
    </div>
  );
}

export function Dialog({ id, open, onClose, title, children, footer, wide = false, restore = true }) {
  const ref = useRef(null);
  const returnTo = useRef(null);
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) {
      returnTo.current = document.activeElement;
      d.showModal();
    } else if (!open && d.open) d.close();
  }, [open]);
  return (
    <dialog
      ref={ref}
      className={`dialog ${wide ? "wide" : ""}`}
      aria-labelledby={`${id}-title`}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      onClose={() => {
        if (open) onClose();
        const el = returnTo.current;
        if (restore && el && document.contains(el)) el.focus();
      }}
    >
      {open && (
        <>
          <header className="dialog-head">
            <h2 id={`${id}-title`}>{title}</h2>
            <button type="button" className="icon-btn" aria-label="Close" onClick={onClose}>
              <X size={16} />
            </button>
          </header>
          <div className="dialog-body">{children}</div>
          {footer && <footer className="dialog-foot">{footer}</footer>}
        </>
      )}
    </dialog>
  );
}

export function Status({ status }) {
  return (
    <span className={`status st-${status}`}>
      <span className="dot" aria-hidden="true" />
      {STATUS[status]}
    </span>
  );
}

export function Notice({ text }) {
  return (
    <p role="status" className={`notice ${text ? "" : "empty"}`}>
      {text}
    </p>
  );
}

export function ModelSelect({ id, value, onChange, defaultLabel, label = "Model", catalog = MODELS, ...props }) {
  return (
    <select {...props} id={id} aria-label={label} value={value ?? ""} onChange={(e) => onChange(e.target.value || null)}>
      {defaultLabel && <option value="">Use project default ({defaultLabel})</option>}
      {!defaultLabel && <option value="">Choose a model</option>}
      {!catalog.length && <option value="">No verified models available</option>}
      {value && !catalog.some((m) => m.id === value) && <option value={value} disabled>{value} — unavailable</option>}
      {catalog.map((m) => (
        <option key={m.id} value={m.id} disabled={!m.available}>
          {m.provider} {m.name}
          {m.available ? "" : ` — ${m.note}`}
        </option>
      ))}
    </select>
  );
}

export function EffortSelect({ id, value, onChange, modelId, defaultLabel, label = "Effort", catalog, ...props }) {
  const m = catalog ? catalog.find((item) => item.id === modelId) : model(modelId);
  const efforts = catalog ? (m?.efforts ?? []).map((id) => ({ id, label: id })) : EFFORTS;
  return (
    <select {...props} id={id} aria-label={label} value={value ?? ""} onChange={(e) => onChange(e.target.value || null)}>
      {defaultLabel && <option value="">Use project default ({defaultLabel})</option>}
      {value && !efforts.some((e) => e.id === value) && <option value={value} disabled>{value} — unavailable</option>}
      {efforts.map((e) => {
        const ok = m?.efforts.includes(e.id);
        return (
          <option key={e.id} value={e.id} disabled={!ok && value !== e.id}>
            {e.label}
            {ok ? "" : ` — not on ${m?.name ?? "this model"}`}
          </option>
        );
      })}
    </select>
  );
}

export function ModeSelect({ id, value, onChange, defaultLabel, label = "Mode" }) {
  return (
    <select id={id} aria-label={label} value={value ?? ""} onChange={(e) => onChange(e.target.value || null)}>
      {defaultLabel && <option value="">Use project default ({defaultLabel})</option>}
      {MODES.map((m) => (
        <option key={m.id} value={m.id}>
          {m.label}
        </option>
      ))}
    </select>
  );
}

export function ApprovalActions({ runId, runs, decisions, decide, showOpen = true }) {
  const run = runs.find((r) => r.id === runId);
  if (!run) return null;
  if (run.status !== "needs_approval")
    return <p className="muted small">This request is no longer open ({run.status}).</p>;
  if (run.live && !run.approval) return <p className="muted small">Approval evidence is unavailable. No permission has been granted.</p>;
  const d = decisions[runId];
  if (d)
    return (
      <p className="decision small">
        {run.live ? "Approval decision recorded by the service." : `${d === "allow" ? "Allow once" : "Skip task"} noted locally · not sent, no permission granted.`}{" "}
        {showOpen && <a className="text-link" href={`#/runs/${runId}`}>Open run</a>}
      </p>
    );
  return (
    <div className="btn-row">
      <button type="button" className="btn" onClick={() => decide(runId, "allow")}>Allow once</button>
      <button type="button" className="btn" onClick={() => decide(runId, "skip")}>{run.live ? "Deny" : "Skip task"}</button>
      {showOpen && <a className="text-link small" href={`#/runs/${runId}`}>Open run</a>}
    </div>
  );
}
