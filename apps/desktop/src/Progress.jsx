import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { GOAL, NOW, clock, dur, model, taskState, primaryRun, hashId, stageOf } from "./data.js";
import { Notice, ApprovalActions } from "./ui.jsx";

const COLUMNS = ["Backlog", "Ready", "Running", "Blocked", "Review", "Done"];

export function ProgressPage({ view, runs, tasks, reviewStatus, openRun, decisions, decide, notice, goal = GOAL, now = NOW, live = false }) {
  const rows = tasks.map((t) => ({ task: t, state: taskState(t, runs, reviewStatus), run: primaryRun(t.id, runs) }));
  return (
    <div className={`progress view-${view}`} id="progview-panel" role="tabpanel" aria-labelledby={`progview-tab-${view}`}>
      <h1 className="sr-only">Progress · {view}</h1>
      {view === "graph" && <GraphView rows={rows} openRun={openRun} goal={goal} />}
      {view === "board" && <BoardView rows={rows} openRun={openRun} />}
      {view === "radar" && <RadarView runs={runs} tasks={tasks} openRun={openRun} decisions={decisions} decide={decide} notice={notice} now={now} live={live} />}
    </div>
  );
}

const providerOf = (run) => run.live ? "Codex" : model(run.settings.model)?.provider ?? "";

function TaskNode({ row, openRun }) {
  const { task, state, run } = row;
  const inner = (
    <>
      <span className={`state-dot s-${state.toLowerCase()}`} aria-hidden="true" />
      <span className="tn-text">
        <span className="tn-title">{task.title}</span>
        <span className="tn-meta">
          {state}
          {run ? <> · <span className="mono">{run.id}</span> · {providerOf(run)}</> : " · no run yet"}
        </span>
      </span>
    </>
  );
  return run ? (
    <button type="button" className="task-node" onClick={() => openRun(run.id)} aria-label={`${task.title}, ${state}. Open run ${run.id}`}>
      {inner}
    </button>
  ) : (
    <div className="task-node static">{inner}</div>
  );
}

const NODE_H = 48;
const GAP = 6;

function GraphView({ rows, openRun, goal }) {
  const order = ["Running", "Blocked", "Ready", "Review", "Backlog", "Done"];
  const sorted = [...rows].sort((a, b) => order.indexOf(a.state) - order.indexOf(b.state));
  const H = Math.max(NODE_H, sorted.length * NODE_H + (sorted.length - 1) * GAP);
  const active = rows.filter((r) => r.state === "Running").length;
  return (
    <div className="graph-scroll">
      <div className="graph" style={{ "--graph-h": `${H}px` }}>
        <div className="g-node goal">
          <span className="g-kicker">Goal</span>
          <span className="g-title">{goal.title}</span>
          <span className="tn-meta">{rows.length} tasks</span>
        </div>
        <svg className="g-link straight" width="32" height={H} aria-hidden="true">
          <line x1="0" y1={H / 2} x2="32" y2={H / 2} />
        </svg>
        <div className="g-node lead">
          <span className="g-kicker">Lead</span>
          <span className="g-title">Factory lead</span>
          <span className="tn-meta">{goal.leadStatus ?? `Delegating to ${active} agents`}</span>
        </div>
        <svg className="g-link" width="48" height={H} aria-hidden="true">
          {sorted.map((r, i) => {
            const y = i * (NODE_H + GAP) + NODE_H / 2;
            return <path key={r.task.id} d={`M0 ${H / 2} C 24 ${H / 2}, 24 ${y}, 48 ${y}`} />;
          })}
        </svg>
        <ul className="g-tasks" aria-label="Tasks">
          {sorted.map((r) => (
            <li key={r.task.id}><TaskNode row={r} openRun={openRun} /></li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function BoardView({ rows, openRun }) {
  return (
    <div className="board-scroll">
      <div className="board">
        {COLUMNS.map((c) => {
          const items = rows.filter((r) => r.state === c);
          return (
            <section key={c} className="board-col" aria-labelledby={`col-${c}`}>
              <h2 id={`col-${c}`} className="col-title">{c} <span className="count">{items.length}</span></h2>
              <ul>
                {items.map((r) => <li key={r.task.id}><TaskNode row={r} openRun={openRun} /></li>)}
              </ul>
            </section>
          );
        })}
      </div>
    </div>
  );
}

/* ---------- Radar (Sundial) ---------- */

const QUAD = { build: 0, check: 1, waiting: 2, plan: 3 };
const QUAD_LABEL = [
  { id: "build", label: "BUILD", deg: 45 },
  { id: "check", label: "CHECK", deg: 135 },
  { id: "waiting", label: "WAITING", deg: 225 },
  { id: "plan", label: "PLAN", deg: 315 },
];
const RINGS = [{ min: 15, label: "15m" }, { min: 60, label: "1h" }, { min: 240, label: "4h" }];
const LEAD_R = 24;
const BEZEL = 10;

function geometry(d) {
  const R = d / 2;
  const r15 = R * 0.33, r60 = R * 0.56, r240 = R * 0.79, rMax = R - BEZEL - 6;
  const scale = (m) => {
    if (m <= 15) return LEAD_R + (r15 - LEAD_R) * (m / 15);
    if (m <= 60) return r15 + (r60 - r15) * ((m - 15) / 45);
    if (m <= 240) return r60 + (r240 - r60) * ((m - 60) / 180);
    return Math.min(rMax, r240 + (rMax - r240) * ((m - 240) / 1200));
  };
  return { R, scale, rings: [r15, r60, r240] };
}

const polar = (c, r, rad) => [c + r * Math.sin(rad), c - r * Math.cos(rad)];

function RadarView({ runs, tasks, openRun, decisions, decide, notice, now, live }) {
  const marks = useMemo(
    () =>
      runs
        .filter((r) => ["claimed", "running", "checking", "interrupting", "needs_approval", "recovery_required"].includes(r.status))
        .map((r) => {
          const { stage, since } = stageOf(r);
          const task = tasks.find((t) => t.id === r.taskId) ?? { title: r.taskId };
          const age = Math.max(0, Math.floor((now - since) / (live ? 60_000 : 1)));
          const f = 0.22 + 0.56 * hashId(r.taskId);
          return {
            run: r, task, stage, age, waiting: stage === "waiting",
            angle: ((QUAD[stage] + f) * Math.PI) / 2,
            fresh: r.heartbeatAt != null && now - r.heartbeatAt <= (live ? 120_000 : 2),
          };
        }),
    [runs, tasks, now, live],
  );
  const working = marks.filter((m) => !m.waiting);
  const needs = marks.filter((m) => m.waiting);

  return (
    <div className="radar-layout">
      <Dial marks={marks} openRun={openRun} now={now} live={live} />
      <aside className="radar-index" aria-label="Active work">
        <h2 className="index-title">Working</h2>
        {working.length === 0 && <p className="muted small">Nothing running.</p>}
        <ul>
          {working.map((m) => (
            <li key={m.run.id}>
              <button type="button" className="index-row" onClick={() => openRun(m.run.id)}>
                <span className="mk-dot" aria-hidden="true" />
                <span>
                  <span className="ir-title">{m.task.title}</span>
                  <span className="ir-meta">{providerOf(m.run)} · {m.stage} · {dur(m.age)} · <span className="mono">{m.run.id}</span></span>
                </span>
              </button>
            </li>
          ))}
        </ul>
        <h2 className="index-title">Needs you</h2>
        {needs.length === 0 && <p className="muted small">Nothing waiting.</p>}
        <ul>
          {needs.map((m) => (
            <li key={m.run.id}>
              <button type="button" className="index-row" onClick={() => openRun(m.run.id)}>
                <span className="mk-dot amber" aria-hidden="true" />
                <span>
                  <span className="ir-title">{m.task.title}</span>
                  <span className="ir-meta">{m.run.approval ? `Waiting for ${m.run.approval.short}` : "Recovery required"} · {dur(Math.floor(m.age))}</span>
                </span>
              </button>
              {m.run.approval && <div className="index-actions">
                <ApprovalActions runId={m.run.id} runs={runs} decisions={decisions} decide={decide} showOpen={false} />
              </div>}
            </li>
          ))}
        </ul>
        <Notice text={notice} />
      </aside>
    </div>
  );
}

function readColors(el) {
  const cs = getComputedStyle(el);
  const v = (n) => cs.getPropertyValue(n).trim();
  return { ring: v("--decorative-ring"), sep: v("--separator"), graphite: v("--action"), amber: v("--amber"), muted: v("--muted") };
}

// shortcut: fixtures avoid label collisions; add placement rules before displaying many live runs.
function Dial({ marks, openRun, now, live }) {
  const wrap = useRef(null);
  const canvas = useRef(null);
  const [{ d, margin }, setDims] = useState({ d: 520, margin: 64 });

  useLayoutEffect(() => {
    const el = wrap.current;
    if (!el) return;
    const measure = () => {
      const w = el.clientWidth - 32;
      const h = el.clientHeight;
      const m = w < 520 ? 40 : 64;
      const caption = el.querySelector("figcaption")?.getBoundingClientRect().height ?? 44;
      const size = Math.min(640, w - m * 2, h > 200 ? h - m * 2 - caption - 8 : Infinity);
      setDims({ d: Math.max(220, Math.floor(size)), margin: m });
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const S = d + margin * 2;
  const c = S / 2;
  const g = geometry(d);

  useEffect(() => {
    const cv = canvas.current;
    if (!cv) return;
    const dpr = window.devicePixelRatio || 1;
    cv.width = S * dpr;
    cv.height = S * dpr;
    const ctx = cv.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, S, S);
    const col = readColors(cv);
    const { R } = g;
    const circle = (r, stroke, w = 1) => {
      ctx.beginPath();
      ctx.arc(c, c, r, 0, Math.PI * 2);
      ctx.strokeStyle = stroke;
      ctx.lineWidth = w;
      ctx.stroke();
    };

    circle(R, col.ring);
    circle(R - BEZEL, col.ring);
    for (let i = 0; i < 120; i++) {
      const a = (i / 120) * Math.PI * 2;
      const len = i % 5 === 0 ? 7 : 4;
      const [x1, y1] = polar(c, R - 1, a);
      const [x2, y2] = polar(c, R - 1 - len, a);
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.strokeStyle = col.ring;
      ctx.lineWidth = 1;
      ctx.stroke();
    }
    ctx.beginPath();
    ctx.moveTo(c, c - (R - BEZEL));
    ctx.lineTo(c, c + (R - BEZEL));
    ctx.moveTo(c - (R - BEZEL), c);
    ctx.lineTo(c + (R - BEZEL), c);
    ctx.strokeStyle = col.sep;
    ctx.stroke();
    g.rings.forEach((r) => circle(r, col.ring));

    for (const m of marks) {
      const r = g.scale(m.age);
      const ux = Math.sin(m.angle), uy = -Math.cos(m.angle);
      const nx = -uy, ny = ux;
      const bx = c + ux * LEAD_R, by = c + uy * LEAD_R;
      const tx = c + ux * r, ty = c + uy * r;
      const wedge = () => {
        ctx.beginPath();
        ctx.moveTo(bx + nx * 5, by + ny * 5);
        ctx.lineTo(tx + nx * 1, ty + ny * 1);
        ctx.lineTo(tx - nx * 1, ty - ny * 1);
        ctx.lineTo(bx - nx * 5, by - ny * 5);
        ctx.closePath();
      };
      if (m.waiting) {
        ctx.save();
        wedge();
        ctx.fillStyle = "rgba(185,122,46,0.10)";
        ctx.fill();
        ctx.clip();
        ctx.strokeStyle = "rgba(185,122,46,0.55)";
        ctx.lineWidth = 0.75;
        for (let t = 0; t < r; t += 3) {
          const px = bx + ux * t, py = by + uy * t;
          ctx.beginPath();
          ctx.moveTo(px + nx * 8, py + ny * 8);
          ctx.lineTo(px - nx * 8 + ux * 4, py - ny * 8 + uy * 4);
          ctx.stroke();
        }
        ctx.restore();
        const [ax, ay] = polar(c, r + 5, m.angle);
        const [ex, ey] = polar(c, R - BEZEL, m.angle);
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(ex, ey);
        ctx.strokeStyle = col.amber;
        ctx.lineWidth = 1;
        ctx.stroke();
        const [n1x, n1y] = polar(c, R - BEZEL - 2, m.angle);
        const [n2x, n2y] = polar(c, R + 3, m.angle);
        ctx.beginPath();
        ctx.moveTo(n1x, n1y);
        ctx.lineTo(n2x, n2y);
        ctx.lineWidth = 2.5;
        ctx.stroke();
      } else {
        const grad = ctx.createLinearGradient(bx, by, tx, ty);
        grad.addColorStop(0, "rgba(59,56,51,0.30)");
        grad.addColorStop(1, "rgba(59,56,51,0.12)");
        wedge();
        ctx.fillStyle = grad;
        ctx.fill();
      }
    }
  }, [marks, S, d]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <figure className="dial-wrap" ref={wrap} aria-labelledby="dial-caption">
      <div className="dial" style={{ width: S, height: S }}>
        <canvas ref={canvas} style={{ width: S, height: S }} aria-hidden="true" />
        {QUAD_LABEL.map((q) => {
          const [x, y] = polar(c, g.R + 22, (q.deg * Math.PI) / 180);
          return <span key={q.id} className="quad-label" style={{ left: x, top: y }} aria-hidden="true">{q.label}</span>;
        })}
        {RINGS.map((rg, i) => (
          <span key={rg.label} className="ring-label" style={{ left: c, top: c - g.rings[i] }} aria-hidden="true">{rg.label}</span>
        ))}
        <div className="lead-disc" style={{ left: c, top: c }} aria-hidden="true">
          <span>Lead</span>
          <span className="lead-state">idle</span>
        </div>
        {marks.map((m) => {
          const r = g.scale(m.age);
          const [x, y] = polar(c, r, m.angle);
          const flip = x > c + g.R * 0.35;
          return (
            <button
              key={m.run.id}
              type="button"
              className={`mark ${m.waiting ? "waiting" : ""} ${m.fresh ? "" : "stale"} ${flip ? "flip" : ""}`}
              style={{ left: x, top: y }}
              onClick={() => openRun(m.run.id)}
              aria-label={`${m.task.title}, ${m.stage}, ${providerOf(m.run)}, ${dur(m.age)} in stage${m.fresh ? "" : ", heartbeat stale"}. Open run ${m.run.id}`}
            >
              <span className="mk-hit"><span className="mk-dot" /></span>
              <span className="mk-text">
                <span className="mk-title">{m.task.short}</span>
                <span className="mk-meta">{m.waiting ? dur(m.age) : `${providerOf(m.run)} · ${dur(m.age)}`}</span>
              </span>
            </button>
          );
        })}
        {marks.filter((m) => m.waiting).map((m) => {
          const [x, y] = polar(c, g.R + 8, m.angle);
          const left = x < c;
          return (
            <span key={`n-${m.run.id}`} className={`notch-label ${left ? "left" : ""}`} style={{ left: x, top: y }} aria-hidden="true">
              {m.run.approval?.short ?? "Recovery"}
            </span>
          );
        })}
      </div>
      <figcaption id="dial-caption" className="dial-caption">
        Distance from centre = time in current stage (15m, 1h, 4h rings). Active work only · {live ? "observed locally" : `sample clock ${clock(now)}`}.
      </figcaption>
    </figure>
  );
}

