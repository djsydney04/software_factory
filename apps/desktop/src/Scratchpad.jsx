import { useEffect, useRef, useState } from "react";
import { Notice } from "./ui.jsx";
import { CADENCES, applyDocumentEdit, completedDocumentRanges, pendingDocumentRanges, reviewCron, reviewLabel, validateReviewConfig } from "./scratchpad.js";

export function ScratchpadPage({ value, setValue, requests, runs, storeError, noteId }) {
  const input = useRef(null);
  const mirror = useRef(null);
  const edit = useRef(null);
  const history = useRef({ past: [], future: [] });
  const text = value.document.text;
  const completed = completedDocumentRanges(value, requests, runs);
  const source = value.document.anchors.find((anchor) => anchor.noteId === noteId);
  const pieces = [];
  let offset = 0;
  for (const range of completed) {
    pieces.push(text.slice(offset, range.start));
    pieces.push(<mark key={range.noteId} className="scratch-completed">{text.slice(range.start, range.end)}</mark>);
    offset = range.end;
  }
  pieces.push(text.slice(offset), "\n");
  useEffect(() => {
    if (!noteId) return;
    input.current?.focus();
    if (source) input.current?.setSelectionRange(source.start, source.end);
  }, [noteId]);
  useEffect(() => {
    const field = input.current;
    let pending = null;
    const clear = () => { pending = null; };
    const beforeInput = (event) => {
      pending = null;
      let start = field.selectionStart;
      let end = field.selectionEnd;
      if (!event.inputType?.startsWith("insert") && !event.inputType?.startsWith("delete")) return;
      if (start === end && event.inputType === "deleteContentBackward") start -= Array.from(field.value.slice(0, start)).at(-1)?.length ?? 0;
      else if (start === end && event.inputType === "deleteContentForward") end += Array.from(field.value.slice(end))[0]?.length ?? 0;
      else if (start === end && event.inputType.startsWith("delete")) return;
      pending = { before: field.value, start, end, inputType: event.inputType };
    };
    const onInput = (event) => { edit.current = pending?.inputType === event.inputType ? pending : null; pending = null; };
    field.addEventListener("beforeinput", beforeInput);
    field.addEventListener("input", onInput);
    field.addEventListener("keydown", clear);
    field.addEventListener("pointerdown", clear);
    field.addEventListener("blur", clear);
    return () => {
      field.removeEventListener("beforeinput", beforeInput);
      field.removeEventListener("input", onInput);
      field.removeEventListener("keydown", clear);
      field.removeEventListener("pointerdown", clear);
      field.removeEventListener("blur", clear);
    };
  }, []);

  return <section className="scratchpad" aria-labelledby="scratchpad-title">
    <header className="scratchpad-head"><h1 id="scratchpad-title" className="pane-title">Scratchpad</h1></header>
    <div className="scratch-document-scroll">
      <div className="scratch-document-editor">
        <div ref={mirror} className="scratch-document-mirror" aria-hidden="true">{pieces}</div>
        <textarea ref={input} aria-label="Scratchpad document" aria-describedby="scratchpad-document-status" spellCheck={false} placeholder="Write anything…" value={text} onChange={(e) => {
          const change = edit.current;
          edit.current = null;
          const result = applyDocumentEdit(value, e.target.value, { history: history.current, change, inputType: e.nativeEvent.inputType });
          history.current = result.history;
          setValue((s) => ({ ...s, document: result.value.document }));
        }} onScroll={(e) => { if (mirror.current) { mirror.current.scrollTop = e.currentTarget.scrollTop; mirror.current.scrollLeft = e.currentTarget.scrollLeft; } }} />
      </div>
    </div>
    <footer className="scratch-document-foot">
      <p id="scratchpad-document-status" className={`${storeError ? "error" : "muted"} small`} role="status">{storeError ? "Couldn’t save locally. Keep this page open and copy your text before reloading." : "Saved locally"}{completed.length > 0 && <span className="sr-only">. {completed.length} {completed.length === 1 ? "passage completed" : "passages completed"} by the agent, highlighted green.</span>}</p>
      {noteId && !source && <p className="muted small" role="status">Source text has changed. Its request keeps the original.</p>}
      {completed.length > 0 && <details className="scratch-completion-details"><summary>{completed.length} completed by agent</summary><ul>{completed.map((range) => <li key={range.noteId}><a className="text-link" href={`#/runs/${encodeURIComponent(range.runId)}`}>{text.slice(range.start, range.end)} <span className="muted">· {range.runId}</span></a></li>)}</ul></details>}
    </footer>
  </section>;
}

export function ScratchpadSettings({ value, setValue, onTriage, notice, storeError, live = false }) {
  const [config, setConfig] = useState(value.config);
  const [error, setError] = useState("");
  const [saved, setSaved] = useState("");
  const dirty = JSON.stringify(config) !== JSON.stringify(value.config);
  const pending = pendingDocumentRanges(value).length + value.document.anchors.filter((anchor) => value.notes.some((note) => note.id === anchor.noteId && note.status === "inbox")).length;
  return <div className="scratch-settings">
    <h2 className="section-title">Scratchpad review</h2>
    <p className="help">{reviewLabel(value.config)}</p>
    <form className="scratch-config" onSubmit={(e) => {
      e.preventDefault();
      const problem = validateReviewConfig(config);
      setError(problem ?? "");
      if (problem) return;
      setValue((s) => ({ ...s, config: structuredClone(config) }));
      setSaved("Review plan saved locally.");
    }}>
      <div className="scratch-config-fields">
        <label><span>Cadence</span><select aria-label="Review cadence" value={config.cadence} onChange={(e) => { setConfig((c) => ({ ...c, cadence: e.target.value })); setSaved(""); }}>{CADENCES.map((c) => <option key={c.id} value={c.id}>{c.label}</option>)}</select></label>
        {config.cadence === "daily" && <label><span>Review time</span><input type="time" aria-label="Review time" value={config.time} onChange={(e) => { setConfig((c) => ({ ...c, time: e.target.value })); setSaved(""); }} /></label>}
        <label className="scratch-zone"><span>IANA time zone</span><input aria-label="Review time zone" value={config.timeZone} onChange={(e) => { setConfig((c) => ({ ...c, timeZone: e.target.value })); setSaved(""); }} aria-invalid={!!error} aria-describedby={error ? "scratch-config-error" : undefined} /></label>
      </div>
      <label className="check"><input type="checkbox" checked={config.enabled} onChange={(e) => { setConfig((c) => ({ ...c, enabled: e.target.checked })); setSaved(""); }} /><span>Enable review plan</span></label>
      <p className="muted small">Cron <code>{reviewCron(config) ?? "Invalid configuration"}</code> · interpreted in the selected zone.</p>
      {error && <p id="scratch-config-error" className="error small" role="alert">{error}</p>}
      <div className="btn-row"><button type="submit" className="btn sm" disabled={!dirty}>Save review plan</button><button type="button" className="btn ghost sm" disabled={!dirty} onClick={() => { setConfig(value.config); setError(""); setSaved(""); }}>Cancel</button></div>
      <Notice text={saved} />
    </form>
    <div className="scratch-preview-section">
      <h3 className="section-title">{live ? "Agent triage" : "Local preview"}</h3>
      <p className="help">{live ? "Recurring Scratchpad triage is not connected yet. This saved plan does not install a cron or submit work. Send a passage as a one-off in Conversation to start a run." : "Runtime disconnected. No agent or cron runs. Preview triage saves new document passages as requests for the lead, without marking them completed."}</p>
      <p className="help">Separate ideas with a blank line. Editing the document leaves submitted requests unchanged. Completed passages highlight green only when their latest linked attempt succeeds.</p>
      {!live && <button type="button" className="btn sm" disabled={!pending} onClick={onTriage}>Preview triage{pending ? ` (${pending})` : ""}</button>}
      {value.lastPreviewAt && <p className="muted small">Last preview {new Date(value.lastPreviewAt).toLocaleString()}</p>}
      <Notice text={notice} />
      {storeError && <p className="error small" role="alert">Local storage is unavailable. Changes may be lost on reload.</p>}
      <a className="text-link small" href="#/scratchpad">Open Scratchpad →</a>
    </div>
  </div>;
}
