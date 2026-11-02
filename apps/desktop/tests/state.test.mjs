import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_SETTINGS, captureContext, contextStatus, restoreRunHistory } from "../src/data.js";

const settings = (changes = {}) => ({ ...structuredClone(DEFAULT_SETTINGS), ...changes });

test("ignore globs handle ? and treat regex punctuation literally", () => {
  const cfg = settings({ ignore: "???.swift\nreport[1]+(draft).md\nDerivedData/" });
  assert.equal(contextStatus("src/abc.swift", cfg).included, false);
  assert.equal(contextStatus("src/long.swift", cfg).included, true);
  assert.equal(contextStatus("report[1]+(draft).md", cfg).included, false);
  assert.equal(contextStatus("report111draft.md", cfg).included, true);
  assert.equal(contextStatus("src/DerivedData/output.swift", cfg).included, false);
});

test("automatic context respects roots, excludes and AGENTS policy while explicit attachments remain inputs", () => {
  const cfg = settings({ roots: ["src/"], alwaysAgents: false, ignore: "*Secret*", excludedFiles: ["src/Skip.swift"] });
  const files = { "README.md": "", "AGENTS.md": "", "src/Keep.swift": "", "src/Skip.swift": "", "src/Secret.swift": "" };
  const snapshot = captureContext(cfg, ["README.md", "README.md"], files);
  assert.deepEqual(snapshot.contextFiles, ["src/Keep.swift", "README.md"]);
  assert.equal(snapshot.contextFiles.includes("AGENTS.md"), false);
  assert.deepEqual(captureContext({ ...cfg, alwaysAgents: true }, [], files).contextFiles, ["AGENTS.md", "src/Keep.swift"]);
});

test("captured context retains its configuration after project choices change", () => {
  const cfg = settings({ roots: ["src/"], excludedFiles: ["src/Skip.swift"], ignore: "*.md", alwaysAgents: false });
  const snapshot = captureContext(cfg, [], { "src/Keep.swift": "", "src/Skip.swift": "" });
  cfg.roots.push("docs/");
  cfg.excludedFiles.push("src/Keep.swift");
  cfg.ignore = "*";
  cfg.alwaysAgents = true;
  assert.deepEqual(snapshot.roots, ["src/"]);
  assert.deepEqual(snapshot.excludedFiles, ["src/Skip.swift"]);
  assert.equal(snapshot.ignore, "*.md");
  assert.equal(snapshot.alwaysAgents, false);
  assert.deepEqual(snapshot.contextFiles, ["src/Keep.swift"]);
});

test("reload preserves stable attempt history and completes a pending local interruption", () => {
  const saved = [
    { id: "R-original", taskId: "T-same", attempt: 1, status: "interrupted", activity: [] },
    { id: "R-uuid", taskId: "T-same", attempt: 2, status: "queued", activity: [] },
    { id: "R-stopping", taskId: "T-other", attempt: 1, status: "interrupting", activity: [] },
  ];
  const restored = restoreRunHistory(JSON.parse(JSON.stringify(saved)));
  assert.deepEqual(restored.slice(0, 2), saved.slice(0, 2));
  assert.equal(restored[2].status, "interrupted");
  assert.match(restored[2].failure, /No real process was stopped/);
  assert.equal(saved[2].status, "interrupting");
});
