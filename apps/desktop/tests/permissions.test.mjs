import assert from "node:assert/strict";
import test from "node:test";
import { normalizeAccess, normalizePermissions, resolvePermissions, validCapturedPermissions, permissionLabel } from "../src/permissions.js";
import { createRequest, restoreRequests } from "../src/scheduling.js";
import { restoreRunHistory } from "../src/data.js";

const project = { model: "codex-gpt-5.3", effort: "medium", mode: "auto", access: "worktree", permissions: { commands: "allow", network: "deny", integrations: "ask" } };

test("invalid incoming permissions never grant Allow; next-send can only narrow the project ceiling", () => {
  assert.deepEqual(normalizePermissions({ commands: "ALLOW", network: true, integrations: null }), { commands: "ask", network: "ask", integrations: "ask" });
  assert.equal(normalizeAccess("full"), "read-only");
  const resolved = resolvePermissions({ ...project, access: "read-only" }, { access: "worktree", permissions: { commands: "ask", network: "allow", integrations: "allow" } });
  assert.deepEqual(resolved, { access: "read-only", permissions: { commands: "ask", network: "deny", integrations: "ask" } });
  assert.equal(project.permissions.commands, "allow");
  assert.equal(validCapturedPermissions({ commands: "allow" }), false);
});

test("legacy run and request evidence keeps missing permissions uncaptured after restore", () => {
  const oldSettings = { model: project.model, effort: project.effort, mode: "ask", access: "worktree" };
  const request = createRequest({ id: "REQ-OLD", text: "Keep old evidence", policy: "lead", settings: oldSettings, context: { contextFiles: [] }, now: 1_800_000_000_000 }).request;
  assert.equal(restoreRequests([request])[0].settings.permissions, undefined);
  const oldRun = { id: "R-OLD", settings: oldSettings, status: "succeeded" };
  assert.equal(restoreRunHistory([oldRun])[0].settings.permissions, undefined);
  assert.equal(permissionLabel(undefined), "Not captured");
  assert.deepEqual(restoreRequests([{ ...request, settings: { ...oldSettings, permissions: { commands: "allow" } } }]), []);
});

test("new request permissions are an independent snapshot, even when defaults change", () => {
  const settings = structuredClone(project);
  const request = createRequest({ id: "REQ-NEW", text: "Inspect only", policy: "lead", settings, context: { contextFiles: [] }, now: 1_800_000_000_000 }).request;
  settings.permissions.network = "allow";
  settings.access = "read-only";
  assert.equal(request.settings.permissions.network, "deny");
  assert.equal(request.settings.access, "worktree");
  assert.equal(request.settings.mode, "auto");
  assert.match(createRequest({ id: "REQ-BAD", text: "Bad permissions", policy: "lead", settings: { ...project, permissions: { commands: true } }, context: { contextFiles: [] } }).error, /valid requested permissions/);
});
