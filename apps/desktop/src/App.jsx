import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowDown, ArrowRight, ArrowUp, CaretDown, ChartBar, ChatCircle, File, FolderOpen, GearSix, Graph, Info, List,
  ListChecks, Lock, LockOpen, Paperclip, Stack, X, FolderSimple, NotePencil, ShieldCheck,
} from "@phosphor-icons/react";
import {
  NOW, GOAL, TASKS, RUNS, LEAD_MESSAGES, REVIEWS, FILES, DEFAULT_SETTINGS, ROOT_OPTIONS, ACTIVE,
  clock, modelLabel, effortLabel, modeLabel, accessLabel, validateCombo, contextStatus, captureContext, restoreRunHistory,
} from "./data.js";
import { Tabs, Dialog, Notice, ModelSelect, EffortSelect, ModeSelect, ApprovalActions } from "./ui.jsx";
import { RunsPage } from "./Runs.jsx";
import { ProgressPage } from "./Progress.jsx";
import { ReviewsPage } from "./Reviews.jsx";
import { restoreReviewState } from "./review-state.js";
import { POLICIES, createRequest, restoreRequests, validateTiming, formatRequestTime, policyLabel, localZone } from "./scheduling.js";
import { normalizePermissions, normalizeAccess, resolvePermissions } from "./permissions.js";
import { PermissionFields, PermissionFacts } from "./Permissions.jsx";
import { ScratchpadPage, ScratchpadSettings } from "./Scratchpad.jsx";
import { restoreScratchpad, previewDocumentTriage } from "./scratchpad.js";
import { RepositoryPreviewNotice } from "./RepositoryIntegration.jsx";

const PAGES = ["conversation", "scratchpad", "runs", "progress", "reviews", "files", "settings"];
// shortcut: browser-local sample state; use the shared service before production execution.
const STORE_KEY = "software-factory.design-preview";
const STORE_VERSION = 1;
const CAROUSEL_FILE = "src/Collection/RecordsCarousel.swift";

function loadStore() {
  try {
    const raw = window.localStorage.getItem(STORE_KEY);
    if (!raw) return { data: null, error: false };
    const v = JSON.parse(raw);
    if (!v || v.version !== STORE_VERSION || typeof v !== "object") return { data: null, error: true };
    return { data: v, error: false };
  } catch {
    return { data: null, error: true };
  }
}

function cleanSettings(s) {
  const merged = { ...DEFAULT_SETTINGS, ...(s && typeof s === "object" ? s : {}) };
  if (!Array.isArray(merged.roots)) merged.roots = DEFAULT_SETTINGS.roots;
  if (!Array.isArray(merged.excludedFiles)) merged.excludedFiles = [];
  if (typeof merged.ignore !== "string") merged.ignore = DEFAULT_SETTINGS.ignore;
  merged.access = normalizeAccess(merged.access);
  merged.permissions = normalizePermissions(merged.permissions);
  return merged;
}

const EMPTY_OVERRIDE = { model: null, effort: null, mode: null };
export const EMPTY_COMPOSER = { draft: "", override: EMPTY_OVERRIDE, permissionOverride: null, attached: [], mode: "message", policy: "lead", localTime: "" };

export function parseHash() {
  const parts = window.location.hash.replace(/^#\/?/, "").split("/");
  const decode = (s) => { try { return s ? decodeURIComponent(s) : null; } catch { return null; } };
  return { page: PAGES.includes(parts[0]) ? parts[0] : "conversation", id: parts[1] === "request" || parts[1] === "schedule" ? null : decode(parts[1]), requestId: parts[1] === "request" ? decode(parts[2]) : null, schedule: parts[1] === "request" || parts[1] === "schedule" };
}

export function App() {
  const stored = useMemo(loadStore, []);
  const [settings, setSettings] = useState(() => cleanSettings(stored.data?.settings));
  const [composer, setComposer] = useState(() => ({
    draft: typeof stored.data?.composer?.draft === "string" ? stored.data.composer.draft : "",
    override: { ...EMPTY_OVERRIDE, ...(stored.data?.composer?.override ?? {}) },
    attached: Array.isArray(stored.data?.composer?.attached) ? stored.data.composer.attached : [],
    mode: stored.data?.composer?.mode === "oneoff" ? "oneoff" : "message",
    policy: POLICIES.some((p) => p.id === stored.data?.composer?.policy) ? stored.data.composer.policy : "lead",
    localTime: typeof stored.data?.composer?.localTime === "string" ? stored.data.composer.localTime : "",
    permissionOverride: stored.data?.composer?.permissionOverride && typeof stored.data.composer.permissionOverride === "object" ? { access: normalizeAccess(stored.data.composer.permissionOverride.access), permissions: normalizePermissions(stored.data.composer.permissionOverride.permissions) } : null,
  }));
  const [fileSaved, setFileSaved] = useState(() => ({ ...FILES, ...(stored.data?.files?.saved ?? {}) }));
  const [fileDrafts, setFileDrafts] = useState(() => ({ ...(stored.data?.files?.drafts ?? {}) }));
  const [storeError, setStoreError] = useState(stored.error);

  const [route, setRoute] = useState(parseHash);
  const [runs, setRunsState] = useState(() => restoreRunHistory(stored.data?.runs));
  const runsRef = useRef(runs);
  const [intake, setIntake] = useState(() => ({ requests: restoreRequests(stored.data?.requests), scratchpad: restoreScratchpad(stored.data?.scratchpad) }));
  const { requests, scratchpad } = intake;
  const setRequests = useCallback((fn) => setIntake((s) => ({ ...s, requests: typeof fn === "function" ? fn(s.requests) : fn })), []);
  const setScratchpad = useCallback((fn) => setIntake((s) => ({ ...s, scratchpad: typeof fn === "function" ? fn(s.scratchpad) : fn })), []);
  const submitBusy = useRef(false);
  const [tasks, setTasks] = useState(() => Array.isArray(stored.data?.tasks) ? stored.data.tasks : TASKS);
  const [messages, setMessages] = useState(() => Array.isArray(stored.data?.messages) ? stored.data.messages : LEAD_MESSAGES);
  const [reviewState, setReviewState] = useState(() => restoreReviewState(stored.data?.reviewState, REVIEWS));
  const reviewStatus = reviewState.status;
  const [reviewsUi, setReviewsUi] = useState({ filter: "all", query: "", lastRow: null, views: {} });
  const [decisions, setDecisions] = useState({});
  const [notices, setNotices] = useState({});
  const [runsUi, setRunsUi] = useState({ filter: "all", query: "", view: "list", tab: "activity", lastRow: null });
  const [progressView, setProgressView] = useState("graph");
  const [fileSel, setFileSel] = useState("README.md");
  const [drawer, setDrawer] = useState(false);
  const [focusComposer, setFocusComposer] = useState(0);

  useEffect(() => {
    const on = () => setRoute(parseHash());
    window.addEventListener("hashchange", on);
    return () => window.removeEventListener("hashchange", on);
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(
        STORE_KEY,
        JSON.stringify({ version: STORE_VERSION, settings, composer, runs, requests, scratchpad, reviewState, tasks, messages, files: { saved: fileSaved, drafts: fileDrafts } }),
      );
      setStoreError(false);
    } catch {
      setStoreError(true);
    }
  }, [settings, composer, runs, requests, scratchpad, reviewState, tasks, messages, fileSaved, fileDrafts]);

  const navigate = useCallback((path) => {
    window.location.hash = `#/${path}`;
  }, []);
  const say = useCallback((where, text) => setNotices((n) => ({ ...n, [where]: text })), []);

  const setRuns = useCallback((fn) => {
    runsRef.current = fn(runsRef.current);
    setRunsState(runsRef.current);
  }, []);
  const updateRun = useCallback((id, fn) => setRuns((rs) => rs.map((r) => (r.id === id ? fn(r) : r))), [setRuns]);
  const nextId = () => {
    let id;
    do { id = window.crypto.randomUUID().slice(0, 8).toUpperCase(); }
    while (runsRef.current.some((run) => run.id === `R-P${id}`));
    return id;
  };

  const nextRun = {
    model: composer.override.model ?? settings.model,
    effort: composer.override.effort ?? settings.effort,
    mode: composer.override.mode ?? settings.mode,
    ...resolvePermissions(settings, composer.permissionOverride),
  };

  const submit = () => {
    const text = composer.draft.trim();
    const error = !text ? "Describe what you want the Factory lead to do." : validateCombo(nextRun.model, nextRun.effort);
    if (error) { say("conversation", error); return; }
    if (submitBusy.current) return;
    let n;
    do { n = window.crypto.randomUUID().slice(0, 8).toUpperCase(); }
    while (requests.some((request) => request.id === `REQ-${n}`) || messages.some((message) => message.id === `m-${n}`));
    const now = Date.now();
    const src = (k) => (composer.override[k] ? "override" : "default");
    const capturedSettings = { ...nextRun, capturedAt: now, source: { model: src("model"), effort: src("effort"), mode: src("mode"), permissions: composer.permissionOverride ? "override" : "default" }, concurrency: settings.concurrency, budget: settings.budget };
    const capturedContext = { ...captureContext(settings, composer.attached, fileSaved), attached: [...composer.attached] };
    let request = null;
    if (composer.mode === "oneoff") {
      const result = createRequest({
        id: `REQ-${n}`, text, policy: composer.policy, localTime: composer.localTime, now,
        settings: capturedSettings,
        context: capturedContext,
      });
      if (result.error) { say("conversation", result.error); return; }
      request = result.request;
    }
    submitBusy.current = true;
    if (request) setRequests((rs) => [...rs, request]);
    setMessages((ms) => [
      ...ms,
      { id: `m-${n}`, role: "you", at: NOW, sentAt: now, text, attached: [...composer.attached], settings: structuredClone(capturedSettings), context: structuredClone(capturedContext), ...(request ? { requestId: request.id, requestMeta: `One-off · ${policyLabel(request)} · ${modelLabel(nextRun.model)} · ${effortLabel(nextRun.effort)}` } : {}) },
      { id: `n-${n}`, role: "receipt", at: NOW, sentAt: now, requestId: request?.id ?? null },
    ]);
    setComposer(EMPTY_COMPOSER);
    setFocusComposer((n) => n + 1);
    say("conversation", "");
    window.setTimeout(() => { submitBusy.current = false; }, 0);
  };

  const newOneOff = () => {
    setComposer((c) => ({ ...c, mode: "oneoff" }));
    setFocusComposer((n) => n + 1);
    navigate("conversation");
  };

  const triage = () => {
    const now = Date.now();
    const capturedSettings = { model: settings.model, effort: settings.effort, mode: settings.mode, ...resolvePermissions(settings), concurrency: settings.concurrency, budget: settings.budget, source: { model: "default", effort: "default", mode: "default", permissions: "default" } };
    const result = previewDocumentTriage(intake, capturedSettings, captureContext(settings, [], fileSaved), { now, makeNoteId: () => `NOTE-${window.crypto.randomUUID().slice(0, 8).toUpperCase()}` });
    if (result.error) { say("scratchpad", result.error); return 0; }
    setIntake({ requests: result.requests, scratchpad: result.scratchpad });
    say("scratchpad", result.created ? `${result.created} ${result.created === 1 ? "request" : "requests"} saved for the Factory lead. The document stays unchanged until work is completed.` : "No new requests. These passages are already linked to the lead.");
    return result.created;
  };

  const retry = (run, cfg) => {
    const siblings = runsRef.current.filter((r) => r.taskId === run.taskId);
    if (siblings.some((r) => ACTIVE.includes(r.status))) return null;
    const id = `R-P${nextId()}`;
    const attempt = Math.max(...siblings.map((r) => r.attempt)) + 1;
    setRuns((rs) => [
      ...rs,
      {
        id, taskId: run.taskId, attempt, status: "queued", startedAt: NOW, updatedAt: NOW, endedAt: null,
        ...(run.requestId ? { requestId: run.requestId } : {}),
        revision: run.revision, stages: [{ stage: "plan", at: NOW }], retriedFrom: run.id, local: true,
        settings: structuredClone({ ...run.settings, ...cfg, capturedAt: NOW }),
        activity: [{ at: NOW, text: `Queued locally as a retry of ${run.id}. Nothing was sent to a provider.` }],
        changed: [], checks: [], artifacts: [],
      },
    ]);
    say("run", `Preview: new run ${id} added · Attempt ${attempt} · queued locally. ${run.id} is unchanged.`);
    return id;
  };

  const interrupt = (id) => {
    const r = runsRef.current.find((x) => x.id === id);
    if (!r || r.status !== "running") return;
    updateRun(id, (x) => ({ ...x, status: "interrupting" }));
    say("run", `Interrupting ${id}…`);
    window.setTimeout(() => {
      updateRun(id, (x) =>
        x.status === "interrupting"
          ? {
              ...x, status: "interrupted", endedAt: NOW, updatedAt: NOW,
              failure: "Interrupted by you in this preview.",
              activity: [...x.activity, { at: NOW, text: "Marked Interrupted locally. No real process was stopped." }],
            }
          : x,
      );
      say("run", `Preview: ${id} interrupted locally. No new run was created.`);
    }, 1200);
  };

  const cancelQueued = (id) => {
    updateRun(id, (x) => (x.status === "queued" ? { ...x, status: "cancelled", endedAt: NOW, updatedAt: NOW } : x));
    say("run", `Preview: ${id} cancelled locally.`);
  };

  const decide = (runId, d) => {
    setDecisions((m) => ({ ...m, [runId]: d }));
    const text = `Preview: “${d === "allow" ? "Allow once" : "Skip task"}” noted locally for ${runId}. No permission was granted.`;
    say("run", text);
    say("radar", text);
    say("conversation", text);
  };

  const toggleAttach = (path) =>
    setComposer((c) => ({
      ...c,
      attached: c.attached.includes(path) ? c.attached.filter((p) => p !== path) : [...c.attached, path],
    }));

  const resetPreview = () => {
    try {
      window.localStorage.removeItem(STORE_KEY);
    } catch {
      /* ignore */
    }
    setSettings(DEFAULT_SETTINGS);
    setComposer(EMPTY_COMPOSER);
    setFileSaved(FILES);
    setFileDrafts({});
    setRuns(() => RUNS);
    setIntake({ requests: [], scratchpad: restoreScratchpad(null) });
    setTasks(TASKS);
    setMessages(LEAD_MESSAGES);
    setReviewState(restoreReviewState(null, REVIEWS));
    setReviewsUi({ filter: "all", query: "", lastRow: null, views: {} });
    setDecisions({});
    setNotices({});
    setRunsUi({ filter: "all", query: "", view: "list", tab: "activity", lastRow: null });
    setProgressView("graph");
    setFileSel("README.md");
    setStoreError(false);
  };

  const readyReviews = REVIEWS.filter((r) => reviewStatus[r.id] === "ready").length;
  const runningCount = runs.filter((r) => r.status === "running" || r.status === "interrupting").length;
  const page = route.page;
  const inRunDetail = page === "runs" && route.id;
  const runView = route.schedule ? "schedule" : runsUi.view;

  let switcher = null;
  if (page === "progress")
    switcher = (
      <Tabs id="progview" label="Progress view" value={progressView} onChange={setProgressView}
        items={[{ id: "graph", label: "Graph" }, { id: "board", label: "Board" }, { id: "radar", label: "Radar" }]} />
    );
  if (page === "runs" && !inRunDetail)
    switcher = (
      <Tabs id="runsview" label="Runs view" value={runView} onChange={(v) => { setRunsUi((u) => ({ ...u, view: v })); navigate(v === "schedule" ? "runs/schedule" : "runs"); }}
        items={[{ id: "list", label: "List" }, { id: "timeline", label: "Timeline" }, { id: "schedule", label: "Schedule", count: requests.filter((r) => r.status !== "cancelled").length || undefined }]} />
    );

  const nav = (
    <Sidebar page={page} readyReviews={readyReviews} projectName={settings.projectName} onNavigate={() => setDrawer(false)} />
  );

  return (
    <div className="app">
      <aside className="sidebar desktop-only" aria-label="Primary">{nav}</aside>
      <Drawer open={drawer} onClose={() => setDrawer(false)}>{nav}</Drawer>
      <div className="main">
        <header className={`topbar ${switcher ? "has-switch" : ""}`}>
          <button type="button" className="icon-btn menu-btn" aria-label="Open navigation" aria-expanded={drawer} onClick={() => setDrawer(true)}>
            <List size={18} />
          </button>
          <p className="goal">
            <span className="goal-title">{GOAL.title}</span>
            <span className="goal-focus"><span aria-hidden="true"> · </span>Now: {GOAL.focus}</span>
          </p>
          <div className="topbar-right">
            {switcher}
            <span className="run-state"><span className="dot" aria-hidden="true" />Running · {runningCount} {runningCount === 1 ? "agent" : "agents"}</span>
            <span className="preview-tag">Design preview</span>
          </div>
        </header>
        <main className={`page page-${page}`} id="main">
          {page === "conversation" && (
            <Conversation
              messages={messages} runs={runs} composer={composer} setComposer={setComposer} settings={settings}
              nextRun={nextRun} submit={submit} toggleAttach={toggleAttach} decisions={decisions} decide={decide}
              notice={notices.conversation} openReview={(id) => navigate(`reviews/${id}`)} focusComposer={focusComposer} storeError={storeError}
            />
          )}
          {page === "runs" && (
            <RunsPage
              runs={runs} tasks={tasks} runId={route.id} ui={{ ...runsUi, view: runView }} setUi={setRunsUi} navigate={navigate}
              settings={settings} retry={retry} interrupt={interrupt} cancelQueued={cancelQueued}
              decisions={decisions} decide={decide} notice={notices.run} say={say}
              requests={requests} setRequests={setRequests} requestId={route.requestId} newOneOff={newOneOff} storeError={storeError}
              openReview={(id) => navigate(`reviews/${id}`)}
            />
          )}
          {page === "scratchpad" && <ScratchpadPage value={scratchpad} setValue={setScratchpad} requests={requests} runs={runs} storeError={storeError} noteId={route.id} />}
          {page === "progress" && (
            <ProgressPage
              view={progressView} runs={runs} tasks={tasks} reviewStatus={reviewStatus}
              openRun={(id) => navigate(`runs/${id}`)} decisions={decisions} decide={decide} notice={notices.radar}
            />
          )}
          {page === "reviews" && (
            <ReviewsPage
              reviewId={route.id} navigate={navigate} runs={runs} tasks={tasks}
              state={reviewState} setState={setReviewState} ui={reviewsUi} setUi={setReviewsUi}
              openRun={(id) => navigate(`runs/${id}`)} storeError={storeError}
            />
          )}
          {page === "files" && (
            <FilesPage
              sel={fileSel} setSel={setFileSel} saved={fileSaved} setSaved={setFileSaved} drafts={fileDrafts}
              setDrafts={setFileDrafts} settings={settings} setSettings={setSettings} attached={composer.attached}
              toggleAttach={toggleAttach}
            />
          )}
          {page === "settings" && (
            <SettingsPage
              saved={settings} setSaved={setSettings} override={composer.override} storeError={storeError}
              resetPreview={resetPreview}
              section={route.id} scratchpad={scratchpad} setScratchpad={setScratchpad} onTriage={triage} scratchpadNotice={notices.scratchpad}
              onSectionChange={(next) => navigate(`settings/${next}`)}
            />
          )}
        </main>
      </div>
    </div>
  );
}

const NAV = [
  { id: "conversation", label: "Conversation", Icon: ChatCircle },
  { id: "scratchpad", label: "Scratchpad", Icon: NotePencil },
  { id: "runs", label: "Runs", Icon: ListChecks },
  { id: "progress", label: "Progress", Icon: Graph },
  { id: "reviews", label: "Reviews", Icon: Stack },
];
const UTIL = [
  { id: "files", label: "Files", Icon: FolderSimple },
  { id: "settings", label: "Settings", Icon: GearSix },
];

export function Sidebar({ page, readyReviews, projectName, onNavigate, live = false }) {
  const item = ({ id, label, Icon }) => (
    <li key={id}>
      <a href={`#/${id}`} className="nav-row" aria-current={page === id ? "page" : undefined} onClick={onNavigate}>
        <Icon size={16} aria-hidden="true" />
        <span>{label}</span>
        {id === "reviews" && readyReviews > 0 && (
          <span className="nav-count" aria-label={`${readyReviews} ready`}>{readyReviews}</span>
        )}
      </a>
    </li>
  );
  return (
    <div className="sidebar-inner">
      <div className="project">
        <span className="project-tile" aria-hidden="true">{projectName.slice(0, 1).toUpperCase()}</span>
        <select aria-label="Project" defaultValue={live ? "live-project" : "green-room"}>
          <option value={live ? "live-project" : "green-room"}>{projectName}</option>
          <option value="more" disabled>{live ? "One project in this local alpha" : "Other projects aren't in this preview"}</option>
        </select>
        <CaretDown size={12} aria-hidden="true" className="project-caret" />
      </div>
      <nav aria-label="Destinations">
        <ul className="nav-list">{NAV.map(item)}</ul>
      </nav>
      <nav aria-label="Project utilities" className="nav-bottom">
        <ul className="nav-list">{UTIL.map(item)}</ul>
      </nav>
    </div>
  );
}

export function Drawer({ open, onClose, children }) {
  const ref = useRef(null);
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    else if (!open && d.open) d.close();
  }, [open]);
  useEffect(() => {
    const mq = window.matchMedia("(min-width: 769px)");
    const on = () => mq.matches && onClose();
    mq.addEventListener("change", on);
    return () => mq.removeEventListener("change", on);
  }, [onClose]);
  return (
    <dialog
      ref={ref}
      className="drawer"
      aria-label="Navigation"
      onCancel={(e) => { e.preventDefault(); onClose(); }}
      onClose={() => { if (open) onClose(); document.querySelector(".menu-btn")?.focus(); }}
      onClick={(e) => { if (e.target === ref.current) onClose(); }}
    >
      <div className="drawer-panel">
        <button type="button" className="icon-btn drawer-close" aria-label="Close navigation" onClick={onClose}>
          <X size={16} />
        </button>
        {children}
      </div>
    </dialog>
  );
}

/* ---------------- Conversation ---------------- */

export function Conversation({ messages, runs, composer, setComposer, settings, nextRun, submit, toggleAttach, decisions, decide, notice, openReview, focusComposer, storeError, live = null, reviews = REVIEWS }) {
  const [pickerOpen, setPickerOpen] = useState(false);
  const [permissionsOpen, setPermissionsOpen] = useState(false);
  const [permissionDraft, setPermissionDraft] = useState(() => resolvePermissions(settings, composer.permissionOverride));
  const input = useRef(null);
  const thread = useRef(null);
  const previousCount = useRef(0);
  const previousFocus = useRef(focusComposer);
  const followUpdates = useRef(true);
  const [hasNewMessages, setHasNewMessages] = useState(false);
  const pickerBtn = useRef(null);
  const pickerPanel = useRef(null);
  const err = live ? live.validate(nextRun.model, nextRun.effort) : validateCombo(nextRun.model, nextRun.effort);
  const overrides = Object.entries(composer.override).filter(([, v]) => v).map(([k]) => k);
  const setO = (k, v) => setComposer((c) => ({ ...c, override: { ...c.override, [k]: v } }));
  const canSend = composer.draft.trim() && (!err || (live && composer.mode === "message")) && !live?.busy;
  const timing = validateTiming(composer.policy, composer.localTime);

  useEffect(() => {
    if (focusComposer) input.current?.focus();
  }, [focusComposer]);

  useEffect(() => {
    const firstVisit = previousCount.current === 0;
    const ownSend = previousFocus.current !== focusComposer;
    const newMessages = previousCount.current !== messages.length;
    if (firstVisit || ownSend || (newMessages && followUpdates.current)) {
      thread.current?.scrollTo({ top: thread.current.scrollHeight, behavior: firstVisit || window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "instant" : "smooth" });
      setHasNewMessages(false);
    } else if (newMessages) setHasNewMessages(true);
    previousCount.current = messages.length;
    previousFocus.current = focusComposer;
  }, [messages.length, focusComposer]);

  useEffect(() => {
    if (err && (!live || composer.mode === "oneoff")) setPickerOpen(true);
  }, [err, composer.mode]);

  useEffect(() => {
    if (pickerOpen) pickerPanel.current?.querySelector("select")?.focus();
  }, [pickerOpen]);

  return (
    <div className="convo">
      <h1 className="sr-only">Conversation with the Factory lead</h1>
      <div className="thread" ref={thread} onScroll={(e) => {
        const el = e.currentTarget;
        followUpdates.current = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
        if (followUpdates.current) setHasNewMessages(false);
      }}>
        <ol className="messages">
          {live && !messages.length && <li className="muted small live-empty-thread">Give the lead direction here, or send a one-off to plan and run. Nothing has been submitted yet.</li>}
          {messages.map((m, i) => {
            if (m.role === "you")
              return (
                <li key={m.id} className="msg you">
                  <p className="meta">You · {m.sentAt ? new Date(m.sentAt).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" }) : clock(m.at)}</p>
                  <div className="bubble">
                    {m.text}
                    {m.attached?.length > 0 && <p className="attached-line">Context: {m.attached.join(", ")}</p>}
                    {m.requestMeta && <p className="attached-line">{m.requestMeta}</p>}
                    {m.settings && ["model", "effort", "mode", "access"].every((key) => typeof m.settings[key] === "string") && !m.requestId && <details className="message-settings"><summary>Requested settings</summary><dl className="request-facts"><dt>Model</dt><dd>{modelLabel(m.settings.model)}</dd><dt>Effort</dt><dd>{effortLabel(m.settings.effort)}</dd><dt>Mode</dt><dd>{modeLabel(m.settings.mode)}</dd><dt>Agent edits</dt><dd>{accessLabel(m.settings.access)}</dd><PermissionFacts permissions={m.settings.permissions} /></dl><p className="muted small">Captured at send · read only</p><ul className="message-context mono">{Array.isArray(m.context?.contextFiles) && m.context.contextFiles.filter((path) => typeof path === "string").map((path) => <li key={path}>{path}</li>)}</ul></details>}
                  </div>
                </li>
              );
            if (m.role === "receipt") return (
              <li key={m.id} className="msg receipt" role="status">
                <span>{live ? m.text : storeError ? "Kept in memory only · local storage unavailable." : m.requestId ? "Saved locally for the lead · not started." : "Message saved locally · not sent to a provider."}</span>
                {m.requestId && <a className="text-link" href={`#/runs/request/${m.requestId}`}>Open in Schedule <ArrowRight size={12} aria-hidden="true" /></a>}
              </li>
            );
            if (m.role === "note") {
              const r = runs.find((x) => x.id === m.runId);
              if (!r) return null;
              return (
                <li key={m.id} className="msg note">
                  <Info size={14} aria-hidden="true" />
                  <span>
                    Preview: queued <span className="mono">{m.runId}</span> · {modelLabel(r.settings.model)} · {effortLabel(r.settings.effort)} · {modeLabel(r.settings.mode)}. No model was called.
                  </span>
                  <a className="text-link" href={`#/runs/${m.runId}`}>Open run</a>
                </li>
              );
            }
            const review = reviews.find((item) => item.id === m.reviewId) ?? (!live && m.review ? reviews[0] : null);
            const reviewRun = review ? runs.find((r) => r.id === review.runId) : null;
            return (
              <li key={m.id} className={`msg lead ${messages[i - 1]?.role === "lead" ? "cont" : ""} ${m.approval ? "ask" : ""}`}>
                <span className="avatar" aria-hidden="true"><ChartBar size={14} /></span>
                <div className="msg-body">
                  <p className="meta"><span className={`who ${messages[i - 1]?.role === "lead" ? "sr-only" : ""}`}>Factory</span> {clock(m.at)}</p>
                  {m.link === "progress" ? <details className="agent-update">
                    <summary>Goal and assignments</summary>
                    <p>{m.text}</p>
                    <a className="text-link" href="#/progress">Open Progress <ArrowRight size={12} aria-hidden="true" /></a>
                  </details> : <p>{m.text}</p>}
                  {review && (
                    <a className="review-link" href={`#/reviews/${review.id}`} onClick={() => openReview(review.id)}>
                      <Stack size={16} aria-hidden="true" />
                      <span className="rl-copy"><span className="rl-title">{live ? review.title : GRID_TITLE}</span><span className="rl-meta mono" title={reviewRun?.revision}>Review #{review.number} · {live && reviewRun?.revision ? reviewRun.revision.slice(0, 8) : reviewRun?.revision ?? "Source unavailable"}</span></span>
                      <span className="rl-open">Open in Reviews <ArrowRight size={12} aria-hidden="true" /></span>
                    </a>
                  )}
                  {m.approval && (
                    <ApprovalActions runId={m.approval} runs={runs} decisions={decisions} decide={decide} />
                  )}
                </div>
              </li>
            );
          })}
        </ol>
      </div>

      <div className="composer-wrap">
        {hasNewMessages && <button type="button" className="btn sm latest-updates" onClick={() => {
          thread.current?.scrollTo({ top: thread.current.scrollHeight, behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "instant" : "smooth" });
          input.current?.focus({ preventScroll: true });
          setHasNewMessages(false);
        }}>Latest updates <ArrowDown size={13} aria-hidden="true" /></button>}
        {storeError && <p role="alert" className="error small">Local storage is unavailable. Changes are kept in memory and may be lost on reload.</p>}
        <Notice text={notice} />
        <form className="composer" onSubmit={(e) => { e.preventDefault(); submit(); }}>
          <label htmlFor="composer-input" className="sr-only">{composer.mode === "oneoff" ? "One-off task for the Factory lead" : "Message to the Factory lead"}</label>
          <textarea
            id="composer-input"
            ref={input}
            rows={2}
            placeholder={composer.mode === "oneoff" ? "Describe one task for the lead to schedule" : "Tell Factory what matters next"}
            value={composer.draft}
            onChange={(e) => setComposer((c) => ({ ...c, draft: e.target.value }))}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                submit();
              }
            }}
            aria-describedby={composer.mode === "oneoff" ? "composer-help oneoff-timing-help" : "composer-help"}
          />
          {composer.attached.length > 0 && (
            <ul className="chips" aria-label="Attached context">
              {composer.attached.map((p) => (
                <li key={p} className="chip">
                  <File size={14} aria-hidden="true" />
                  <span className="mono">{p.split("/").pop()}</span>
                  <span className="muted">context only</span>
                  <button type="button" className="chip-x" aria-label={`Remove ${p}`} onClick={() => toggleAttach(p)}>
                    <X size={12} />
                  </button>
                </li>
              ))}
            </ul>
          )}
          {pickerOpen && (
            <fieldset
              className="next-run"
              id="next-run-panel"
              ref={pickerPanel}
              onKeyDown={(e) => {
                if (e.key === "Escape" && !err) {
                  setPickerOpen(false);
                  pickerBtn.current?.focus();
                }
              }}
            >
              <legend>Next send settings <span className="muted">· captured only when you send</span></legend>
              <div className="next-run-grid">
                <label>
                  <span>Model</span>
                  <ModelSelect value={composer.override.model} onChange={(v) => setO("model", v)} catalog={live?.catalog} defaultLabel={modelLabel(settings.model)} aria-invalid={!!err} aria-describedby={err ? "combo-err" : undefined} />
                </label>
                <label>
                  <span>Effort</span>
                  <EffortSelect value={composer.override.effort} onChange={(v) => setO("effort", v)} catalog={live?.catalog} modelId={nextRun.model} defaultLabel={effortLabel(settings.effort)} aria-invalid={!!err} aria-describedby={err ? "combo-err" : undefined} />
                </label>
                {!live && <label>
                  <span>Mode</span>
                  <ModeSelect value={composer.override.mode} onChange={(v) => setO("mode", v)} defaultLabel={modeLabel(settings.mode)} />
                </label>}
              </div>
              <div className="next-run-foot">
                {err ? (
                  <p className="error" id="combo-err" role="alert">{err}</p>
                ) : (
                  <p className="muted">
                    {overrides.length ? `Override: ${overrides.join(", ")}. Everything else follows project defaults.` : "Inherited from project defaults."}{" "}
                    <a className="text-link" href="#/settings">Project defaults</a>
                  </p>
                )}
                <button type="button" className="btn ghost sm" disabled={!overrides.length} onClick={() => setComposer((c) => ({ ...c, override: EMPTY_OVERRIDE }))}>
                  Use project default
                </button>
              </div>
            </fieldset>
          )}
          <div className={`composer-bar${composer.mode === "oneoff" ? " is-oneoff" : ""}`}>
            <div className="composer-tools">
            <button
              type="button"
              className="icon-btn"
              aria-pressed={composer.attached.includes(CAROUSEL_FILE)}
              aria-label={live ? "File attachments unavailable in this alpha" : "Attach RecordsCarousel.swift as context"}
              title={live ? "File attachments are not connected" : "Attach RecordsCarousel.swift as context (no edit access)"}
              onClick={() => toggleAttach(CAROUSEL_FILE)}
              disabled={!!live}
            >
              <Paperclip size={16} />
            </button>
            <button
              type="button"
              ref={pickerBtn}
              className={`picker-btn ${err ? "invalid" : ""}`}
              aria-expanded={pickerOpen}
              aria-controls="next-run-panel"
              aria-label={`Next send settings: ${modelLabel(nextRun.model)}, ${effortLabel(nextRun.effort)}, ${modeLabel(nextRun.mode)}, ${overrides.length ? "override" : "project defaults"}`}
              title={`Next send: ${modelLabel(nextRun.model)} · ${effortLabel(nextRun.effort)} · ${modeLabel(nextRun.mode)} · ${overrides.length ? "Override" : "Project defaults"}`}
              onClick={() => setPickerOpen((o) => !o)}
            >
              <span className="scope">Next send</span>
              <span className="picker-val">{modelLabel(nextRun.model)} · {effortLabel(nextRun.effort)}</span>
              {!live && nextRun.mode !== "ask" && <span className="picker-mode">{modeLabel(nextRun.mode)}</span>}
              {overrides.length > 0 && <span className="src ovr">Override</span>}
              <CaretDown size={12} aria-hidden="true" />
            </button>
            <button type="button" className="permission-btn" aria-haspopup="dialog" onClick={() => { setPermissionDraft(resolvePermissions(settings, composer.permissionOverride)); setPermissionsOpen(true); }} title={`Next-send permissions · ${composer.permissionOverride ? "restricted override" : `${settings.projectName} project defaults`}`}><ShieldCheck size={15} aria-hidden="true" /><span>Permissions</span>{composer.permissionOverride && <span className="permission-override-dot" aria-label="Overridden" />}</button>
            </div>
            <div className="composer-sendrow">
            {composer.mode === "oneoff" && <div className="composer-intent">
              <label className="when-control"><span>When</span><select aria-label="When to schedule" value={composer.policy} onChange={(e) => setComposer((c) => ({ ...c, policy: e.target.value }))}>{POLICIES.map((p) => <option key={p.id} value={p.id}>{p.label}</option>)}</select></label>
              {composer.policy === "at" && <input aria-label="Scheduled local date and time" type="datetime-local" value={composer.localTime} onChange={(e) => setComposer((c) => ({ ...c, localTime: e.target.value }))} aria-describedby="oneoff-timing-help" aria-invalid={!!composer.localTime && !!timing.error} />}
              <p className={`oneoff-help small ${composer.policy === "at" && composer.localTime && timing.error ? "error" : "muted"}`} id="oneoff-timing-help" aria-live="polite">
                {composer.policy === "at" ? timing.error ? `${timing.error} Browser zone: ${localZone()}.` : `${formatRequestTime(timing.scheduledAt, timing.timeZone, true)} · ${timing.timeZone}. Earliest start; the lead fits it around ongoing work.` : composer.policy === "next" ? "After current work and dependencies." : "Factory decides when this fits."}
              </p>
            </div>}
            <span id="composer-help" className="kbd-hint">⌘/Ctrl + Enter</span>
            <select className="message-type" aria-label="Message type" value={composer.mode} onChange={(e) => setComposer((c) => ({ ...c, mode: e.target.value }))}><option value="message">Message</option><option value="oneoff">One-off</option></select>
            <button type="submit" className="send" aria-label={`${composer.mode === "oneoff" ? "Submit one-off" : "Send message"} to Factory lead${live ? "" : " (preview)"}`} title={composer.mode === "oneoff" ? "Submit one-off to Factory lead" : "Send message to Factory lead"} disabled={!canSend}>
              <ArrowUp size={16} weight="bold" />
            </button>
            </div>
          </div>
        </form>
        <Dialog id="next-send-permissions" open={permissionsOpen} onClose={() => setPermissionsOpen(false)} title="Permissions">
          <form onSubmit={(e) => { e.preventDefault(); setComposer((c) => ({ ...c, permissionOverride: structuredClone(permissionDraft) })); setPermissionsOpen(false); }}>
            <section className="permission-project" aria-label="Saved project permissions">
              <div className="permission-project-head"><strong>{settings.projectName} · Project permissions</strong><a className="text-link" href="#/settings/permissions" onClick={() => setPermissionsOpen(false)}>Edit project permissions <ArrowRight size={12} aria-hidden="true" /></a></div>
              <dl className="permission-summary"><dt>Agent edits</dt><dd>{accessLabel(settings.access)}</dd><PermissionFacts permissions={settings.permissions} /></dl>
            </section>
            <p className="scope-line"><strong>Next send</strong> · {composer.permissionOverride ? "Restricted override" : "Inherits project permissions"}</p>
            <PermissionFields value={permissionDraft} ceiling={resolvePermissions(settings)} onChange={setPermissionDraft} />
            <p className="help">Restrict this send here, or edit the project rules above. {live ? "The runtime blocks unsupported restrictions instead of granting broader access." : "Preview only; no system access is granted."}</p>
            <div className="dialog-foot"><button type="button" className="btn ghost" onClick={() => { setComposer((c) => ({ ...c, permissionOverride: null })); setPermissionsOpen(false); }}>Use project defaults</button><span className="spacer" /><button type="button" className="btn" onClick={() => setPermissionsOpen(false)}>Cancel</button><button type="submit" className="btn primary">Use for next send</button></div>
          </form>
        </Dialog>
      </div>
    </div>
  );
}

const GRID_TITLE = "Tighten Collection grid spacing";

/* ---------------- Files ---------------- */

function buildTree(paths) {
  const groups = new Map();
  for (const p of paths) {
    const i = p.lastIndexOf("/");
    const dir = i === -1 ? "" : p.slice(0, i + 1);
    if (!groups.has(dir)) groups.set(dir, []);
    groups.get(dir).push(p);
  }
  return [...groups.entries()].sort(([a], [b]) => (a === "" ? -1 : b === "" ? 1 : a.localeCompare(b)));
}

function FilesPage({ sel, setSel, saved, setSaved, drafts, setDrafts, settings, setSettings, attached, toggleAttach }) {
  const [notice, setNotice] = useState("");
  const [pattern, setPattern] = useState("");
  const paths = Object.keys(FILES);
  const tree = buildTree(paths);
  const text = drafts[sel] ?? saved[sel];
  const dirty = drafts[sel] != null && drafts[sel] !== saved[sel];
  const ctx = contextStatus(sel, settings);
  const patterns = settings.ignore.split("\n").map((s) => s.trim()).filter(Boolean);
  const isAttached = attached.includes(sel);

  const savePreview = () => {
    setSaved((s) => ({ ...s, [sel]: drafts[sel] }));
    setDrafts((d) => {
      const n = { ...d };
      delete n[sel];
      return n;
    });
    setNotice(`Saved ${sel} in this preview only. No file was written.`);
  };
  const updateSettings = (fn, msg) => {
    setSettings((s) => fn(s));
    setNotice(`${msg} Applies to future runs; saved locally.`);
  };

  return (
    <div className="files">
      <nav className="file-tree" aria-label="Project files">
        <h1 className="pane-title">Files</h1>
        <label className="sr-only" htmlFor="file-select">File</label>
        <select id="file-select" className="narrow-only" value={sel} onChange={(e) => setSel(e.target.value)}>
          {paths.map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
        <ul className="tree wide-only">
          {tree.map(([dir, files]) => (
            <li key={dir || "root"}>
              {dir && (
                <p className="tree-dir">
                  <FolderOpen size={14} aria-hidden="true" />
                  <span className="mono">{dir}</span>
                </p>
              )}
              <ul>
                {files.map((p) => {
                  const st = contextStatus(p, settings);
                  return (
                    <li key={p}>
                      <button type="button" className={`tree-file ${dir ? "nested" : ""}`} aria-current={sel === p ? "true" : undefined} onClick={() => setSel(p)}>
                        <File size={14} aria-hidden="true" />
                        <span className="mono tf-name">{p.slice(dir.length)}</span>
                        {drafts[p] != null && drafts[p] !== saved[p] && <span className="tf-flag" aria-label="unsaved">•</span>}
                        {!st.included && <span className="tf-flag muted">excluded</span>}
                      </button>
                    </li>
                  );
                })}
              </ul>
            </li>
          ))}
        </ul>
      </nav>

      <section className="file-main" aria-labelledby="file-path">
        <header className="file-head">
          <h2 id="file-path" className="mono">{sel}</h2>
          <span className={`ctx-label ${ctx.included ? "on" : ""}`}>{ctx.included ? "In context" : "Not in context"}</span>
          <span className="access-label">
            {settings.access === "read-only" ? <Lock size={14} aria-hidden="true" /> : <LockOpen size={14} aria-hidden="true" />}
            Agents: {accessLabel(settings.access)}
          </span>
          <span className="spacer" />
          <span className="muted small" aria-live="polite">{dirty ? "Unsaved changes" : "Saved"}</span>
          <button type="button" className="btn sm" disabled={!dirty} onClick={() => setDrafts((d) => { const n = { ...d }; delete n[sel]; return n; })}>
            Discard
          </button>
          <button type="button" className="btn primary sm" disabled={!dirty} onClick={savePreview}>Save preview</button>
        </header>
        <label className="sr-only" htmlFor="file-editor">Contents of {sel}</label>
        <textarea
          id="file-editor"
          className="code-editor"
          spellCheck={false}
          wrap="off"
          value={text}
          onChange={(e) => setDrafts((d) => ({ ...d, [sel]: e.target.value }))}
        />
        <Notice text={notice} />
      </section>

      <aside className="inspector" aria-label="File settings">
        <section>
          <h2 className="section-title">This file</h2>
          <label className="check">
            <input
              type="checkbox"
              checked={!settings.excludedFiles.includes(sel)}
              disabled={sel === "AGENTS.md" && settings.alwaysAgents}
              onChange={(e) =>
                updateSettings(
                  (s) => ({ ...s, excludedFiles: e.target.checked ? s.excludedFiles.filter((p) => p !== sel) : [...s.excludedFiles, sel] }),
                  e.target.checked ? `${sel} can be used as context again.` : `${sel} excluded from context (not deleted).`,
                )
              }
            />
            <span>Allow in automatic context</span>
          </label>
          <p className="help">{ctx.reason}.</p>
          <label className="check">
            <input type="checkbox" checked={isAttached} onChange={() => toggleAttach(sel)} />
            <span>Attach to next message</span>
          </label>
          <p className="help">An attachment is an input. It doesn't grant edit access.</p>
          <dl className="kv">
            <dt>Agent edits</dt>
            <dd>{accessLabel(settings.access)} <span className="muted">· project</span></dd>
          </dl>
          <a className="text-link small" href="#/settings">Change in Settings</a>
        </section>
        <section>
          <h2 className="section-title">Project context</h2>
          <fieldset className="plain">
            <legend className="field-label">Roots</legend>
            {ROOT_OPTIONS.map((r) => (
              <label key={r.id} className="check">
                <input
                  type="checkbox"
                  checked={settings.roots.includes(r.id)}
                  onChange={(e) =>
                    updateSettings(
                      (s) => ({ ...s, roots: e.target.checked ? [...s.roots, r.id] : s.roots.filter((x) => x !== r.id) }),
                      `${r.label} ${e.target.checked ? "added to" : "removed from"} context roots.`,
                    )
                  }
                />
                <span className="mono">{r.label}</span>
              </label>
            ))}
          </fieldset>
          <p className="field-label">Ignore patterns</p>
          <ul className="patterns">
            {patterns.map((p) => (
              <li key={p}>
                <span className="mono">{p}</span>
                <button
                  type="button"
                  className="icon-btn sm"
                  aria-label={`Remove pattern ${p}`}
                  onClick={() => updateSettings((s) => ({ ...s, ignore: patterns.filter((x) => x !== p).join("\n") }), `Removed ${p}.`)}
                >
                  <X size={12} />
                </button>
              </li>
            ))}
          </ul>
          <form
            className="inline-form"
            onSubmit={(e) => {
              e.preventDefault();
              const p = pattern.trim();
              if (!p || patterns.includes(p)) return;
              updateSettings((s) => ({ ...s, ignore: [...patterns, p].join("\n") }), `Added ${p}.`);
              setPattern("");
            }}
          >
            <label className="sr-only" htmlFor="pattern-input">New ignore pattern</label>
            <input id="pattern-input" className="mono" placeholder="e.g. *.png" value={pattern} onChange={(e) => setPattern(e.target.value)} />
            <button type="submit" className="btn sm" disabled={!pattern.trim()}>Add</button>
          </form>
          <p className="help">Excluding a file never deletes it. Running work keeps its captured context.</p>
        </section>
      </aside>
    </div>
  );
}

/* ---------------- Settings ---------------- */

const SETTING_TABS = [
  { id: "general", label: "General" },
  { id: "models", label: "Models & execution" },
  { id: "permissions", label: "Permissions" },
  { id: "integrations", label: "Integrations" },
  { id: "files", label: "Files & context" },
  { id: "scratchpad", label: "Scratchpad" },
];

function SettingsPage({ saved, setSaved, override, storeError, resetPreview, section, scratchpad, setScratchpad, onTriage, scratchpadNotice, onSectionChange }) {
  const settingsNav = useRef(null);
  const [tab, setTab] = useState(() => SETTING_TABS.some((item) => item.id === section) ? section : "models");
  const [draft, setDraft] = useState(saved);
  const [notice, setNotice] = useState(storeError ? "Saved preview data couldn't be read, so sample defaults are in use." : "");
  const [confirmReset, setConfirmReset] = useState(false);
  const dirty = JSON.stringify(draft) !== JSON.stringify(saved);
  const err = validateCombo(draft.model, draft.effort);
  const set = (k, v) => setDraft((d) => ({ ...d, [k]: v }));
  const nextModel = override.model ?? saved.model;
  const standalone = ["scratchpad", "integrations"].includes(tab);
  const Panel = standalone ? "section" : "form";
  useEffect(() => { if (SETTING_TABS.some((item) => item.id === section)) setTab(section); }, [section]);
  useEffect(() => {
    if (section === "permissions") settingsNav.current?.querySelector("#settings-tab-permissions")?.focus();
  }, []);

  return (
    <div className="settings">
      <div className="settings-nav" ref={settingsNav}>
        <h1 className="pane-title">Settings</h1>
        <Tabs id="settings" label="Settings categories" vertical items={SETTING_TABS} value={tab} onChange={(next) => { setTab(next); onSectionChange(next); }} />
      </div>
      <Panel
        className="settings-form"
        id="settings-panel"
        role="tabpanel"
        aria-labelledby={`settings-tab-${tab}`}
        onSubmit={standalone ? undefined : (e) => {
          e.preventDefault();
          if (err || !dirty) return;
          setSaved(draft);
          setNotice(tab === "permissions" ? "Project permissions saved locally. Future sends inherit these rules; existing runs and requests keep their captured permissions." : "Project defaults saved locally. Existing runs and requests keep their captured settings.");
        }}
      >
        {!standalone && <p className="scope-line">
          <strong>Project defaults</strong> · {saved.projectName} · apply to future sends and runs.
        </p>}

        {tab === "scratchpad" && <ScratchpadSettings value={scratchpad} setValue={setScratchpad} onTriage={onTriage} notice={scratchpadNotice} storeError={storeError} />}
        {tab === "integrations" && <RepositoryPreviewNotice />}

        {tab === "general" && (
          <div className="fields">
            <Field id="s-name" label="Project name">
              <input id="s-name" value={draft.projectName} onChange={(e) => set("projectName", e.target.value)} />
            </Field>
            <Field id="s-branch" label="Base branch" help="Runs branch from here. Sample value only.">
              <input id="s-branch" className="mono" value={draft.baseBranch} onChange={(e) => set("baseBranch", e.target.value)} />
            </Field>
            <Field id="s-reset" label="Preview data" help="Restores sample runs, requests, Scratchpad notes, settings, drafts, reviews and file edits in this browser.">
              <button id="s-reset" aria-label="Reset preview data" type="button" className="btn" onClick={() => setConfirmReset(true)}>Reset preview data…</button>
            </Field>
          </div>
        )}

        {tab === "models" && (
          <div className="fields">
            <Field id="s-model" label="Default model" help="Sample catalog. Gemini 3.1 isn't connected in this preview.">
              <ModelSelect id="s-model" label="Default model" value={draft.model} onChange={(v) => v && set("model", v)} aria-invalid={!!err} aria-describedby={err ? "s-effort-error" : undefined} />
            </Field>
            <Field id="s-effort" label="Effort" error={err}>
              <EffortSelect id="s-effort" label="Effort" value={draft.effort} onChange={(v) => v && set("effort", v)} modelId={draft.model} aria-invalid={!!err} aria-describedby={err ? "s-effort-error" : undefined} />
            </Field>
            <Field id="s-mode" label="Execution mode" help="Workflow preference. Explicit permission denials always take precedence.">
              <ModeSelect id="s-mode" label="Execution mode" value={draft.mode} onChange={(v) => v && set("mode", v)} />
            </Field>
            <Field id="s-conc" label="Concurrent runs">
              <select id="s-conc" value={draft.concurrency} onChange={(e) => set("concurrency", Number(e.target.value))}>
                {[1, 2, 3, 4].map((n) => <option key={n} value={n}>{n}</option>)}
              </select>
            </Field>
            <Field id="s-budget" label="Daily budget (USD)" help="Sample cap. The preview doesn't track spend.">
              <input id="s-budget" type="number" min={0} step={5} value={draft.budget} onChange={(e) => set("budget", Number(e.target.value))} />
            </Field>
            <p className="help resolve">
              Next send resolves to <strong>{modelLabel(nextModel)}</strong> ·{" "}
              {override.model ? "composer override, unaffected by these defaults" : "inherited from saved defaults"}.
            </p>
          </div>
        )}

        {tab === "permissions" && <div className="fields">
          <h2 className="section-title">Project permissions</h2>
          <p className="help">Default rules for every agent in {saved.projectName}. Ask pauses for approval; Deny blocks a capability. Next-send overrides can only restrict these rules.</p>
          <PermissionFields describe value={draft} onChange={(values) => setDraft((d) => ({ ...d, access: values.access, permissions: values.permissions }))} />
          <p className="help">Apply to future sends and runs. Execution mode never bypasses a denial. Saved in this preview; not enforced by an OS or provider yet.</p>
        </div>}

        {tab === "files" && (
          <div className="fields">
            <fieldset className="field plain">
              <legend className="field-label">Context roots</legend>
              {ROOT_OPTIONS.map((r) => (
                <label key={r.id} className="check">
                  <input
                    type="checkbox"
                    checked={draft.roots.includes(r.id)}
                    onChange={(e) => set("roots", e.target.checked ? [...draft.roots, r.id] : draft.roots.filter((x) => x !== r.id))}
                  />
                  <span className="mono">{r.label}</span>
                </label>
              ))}
            </fieldset>
            <Field id="s-ignore" label="Ignore patterns" help="One per line. Ignored files stay on disk.">
              <textarea id="s-ignore" className="mono" rows={4} value={draft.ignore} onChange={(e) => set("ignore", e.target.value)} />
            </Field>
            <label className="check">
              <input type="checkbox" checked={draft.alwaysAgents} onChange={(e) => set("alwaysAgents", e.target.checked)} />
              <span>Always include AGENTS.md</span>
            </label>
            <p className="help">Per-file choices and attachments live in <a className="text-link" href="#/files">Files</a>.</p>
          </div>
        )}

        {!standalone && <footer className="form-foot">
          <Notice text={notice} />
          <span className="spacer" />
          <span className="muted small">{dirty ? "Unsaved changes" : "No changes"}</span>
          <button type="button" className="btn" disabled={!dirty} onClick={() => { setDraft(saved); setNotice("Changes discarded."); }}>
            Cancel
          </button>
          <button type="submit" className="btn primary" disabled={!dirty || !!err}>{tab === "permissions" ? "Save project permissions" : "Save defaults"}</button>
        </footer>}
      </Panel>

      <Dialog
        id="reset"
        open={confirmReset}
        onClose={() => setConfirmReset(false)}
        title="Reset preview data?"
        footer={
          <>
            <button type="button" className="btn" onClick={() => setConfirmReset(false)}>Keep data</button>
            <button
              type="button"
              className="btn primary"
              onClick={() => {
                resetPreview();
                setDraft(DEFAULT_SETTINGS);
                setConfirmReset(false);
                setNotice("Preview data reset to samples.");
              }}
            >
              Reset
            </button>
          </>
        }
      >
        <p>Sample runs, conversation, reviews, defaults and file edits return to their original values. This only clears design-preview data in this browser.</p>
      </Dialog>
    </div>
  );
}

function Field({ id, label, help, error, children }) {
  return (
    <div className="field">
      <label className="field-label" htmlFor={id}>{label}</label>
      {children}
      {error ? <p className="error" id={`${id}-error`} role="alert">{error}</p> : help ? <p className="help">{help}</p> : null}
    </div>
  );
}

