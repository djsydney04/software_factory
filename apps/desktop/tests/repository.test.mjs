import test from "node:test";
import assert from "node:assert/strict";
import { initialRepositoryRead, repositoryCiEvidence, repositoryReadReducer, safeRepositoryUrl } from "../src/repository-state.js";
import { emptySnapshot, validateSnapshot } from "../src/live-state.js";

const connection = { projectId: "project", provider: "github", host: "github.com", path: "owner/repo", connectedAt: "2026-11-01T12:00:00Z" };
const detail = () => ({ change: { headSha: "new-sha" }, ci: { state: "success", revision: "new-sha", complete: true, limitations: [], items: [{ name: "unit tests", state: "success", revision: "new-sha" }] } });

test("provider links stay on the connected repository and reject credentials, redirects and injected schemes", () => {
  assert.equal(safeRepositoryUrl("https://github.com/owner/repo/pull/42", connection), "https://github.com/owner/repo/pull/42");
  assert.equal(safeRepositoryUrl("https://github.com/Owner/Repo/actions/runs/42", connection), "https://github.com/Owner/Repo/actions/runs/42");
  const gitlab = { ...connection, provider: "gitlab", host: "gitlab.com", path: "group/subgroup/repo" };
  assert.equal(safeRepositoryUrl("https://gitlab.com/group/subgroup/repo/-/pipelines/42", gitlab), "https://gitlab.com/group/subgroup/repo/-/pipelines/42");
  for (const url of ["javascript:alert(1)", "http://github.com/owner/repo", "https://gitlab.com/owner/repo", "https://github.com.attacker.example/owner/repo", "https://github.com/owner/repository", "https://github.com/other/repo/pull/1", "https://user:secret@github.com/owner/repo", "https://github.com:444/owner/repo", "https://github.com/owner/repo?token=private", "https://github.com/owner/repo#fragment", "https://github.com/owner/repo/../../other/repo"]) assert.equal(safeRepositoryUrl(url, connection), null, url);
});

test("older page, selection and disconnected responses cannot replace the current repository read", () => {
  let state = repositoryReadReducer(initialRepositoryRead(), { type: "start", requestId: 1, scope: "project/page1" });
  state = repositoryReadReducer(state, { type: "start", requestId: 2, scope: "project/page2" });
  assert.equal(repositoryReadReducer(state, { type: "resolved", requestId: 1, scope: "project/page1", value: { items: ["old"] } }), state);
  state = repositoryReadReducer(state, { type: "resolved", requestId: 2, scope: "project/page2", value: { items: ["current"] } });
  assert.deepEqual(state.value.items, ["current"]);
  state = repositoryReadReducer(state, { type: "start", requestId: 3, scope: "project/request2" });
  assert.equal(repositoryReadReducer(state, { type: "rejected", requestId: 2, scope: "project/page2", error: "late failure" }), state);
  state = repositoryReadReducer(state, { type: "clear" });
  assert.equal(repositoryReadReducer(state, { type: "resolved", requestId: 3, scope: "project/request2", value: detail() }), state);
  assert.equal(state.value, null);
});

test("refreshing the same request clears old evidence and ignores the previous refresh response", () => {
  let state = repositoryReadReducer(initialRepositoryRead(), { type: "start", requestId: 1, scope: "request/7" });
  state = repositoryReadReducer(state, { type: "resolved", requestId: 1, scope: "request/7", value: detail() });
  state = repositoryReadReducer(state, { type: "start", requestId: 2, scope: "request/7" });
  assert.equal(state.value, null);
  assert.equal(state.status, "loading");
  assert.equal(repositoryReadReducer(state, { type: "resolved", requestId: 1, scope: "request/7", value: detail() }), state);
});

test("CI passes only with complete passing checks on the exact request head", () => {
  assert.equal(repositoryCiEvidence(detail()).passed, true);
  for (const mutate of [
    (value) => { value.ci.revision = "older-sha"; },
    (value) => { value.change.headSha = ""; },
    (value) => { value.ci.complete = false; },
    (value) => { value.ci.items = []; },
    (value) => { value.ci.items[0].revision = "older-sha"; },
    (value) => { value.ci.items[0].state = "unknown"; },
    (value) => { value.ci.state = "new-provider-state"; },
  ]) {
    const value = detail(); mutate(value);
    const evidence = repositoryCiEvidence(value);
    assert.equal(evidence.passed, false, JSON.stringify(value));
    assert.notEqual(evidence.state, "success");
    assert.notEqual(evidence.label, "Passed");
  }
});

test("empty, running, cancelled and unknown CI remain distinct from passing evidence", () => {
  for (const [state, label] of [["none", "No checks reported"], ["running", "Running"], ["pending", "Pending"], ["cancelled", "Cancelled"], ["skipped", "Skipped"], ["unknown", "Unknown"], ["failure", "Failed"]]) {
    const value = detail(); value.ci.state = state; value.ci.items[0].state = state;
    if (state === "none") value.ci.items = [];
    const evidence = repositoryCiEvidence(value);
    assert.equal(evidence.label, label);
    assert.equal(evidence.passed, false);
  }
});

test("older native snapshots have no repository connection rather than fixture integration data", () => {
  const snapshot = emptySnapshot(); delete snapshot.repositoryConnections;
  assert.deepEqual(validateSnapshot(snapshot).repositoryConnections, []);
  assert.throws(() => validateSnapshot({ ...snapshot, repositoryConnections: "connected" }), /unsupported snapshot/);
});
