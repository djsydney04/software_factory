#!/usr/bin/python3
"""Bounded, network-free gh/glab API fixture. Configuration lives beside this copy."""
import json
from pathlib import Path
import sys
import time
import os
from urllib.parse import urlsplit

directory = Path(__file__).resolve().parent
config_path = directory / "repository-config.json"
config = json.loads(config_path.read_text()) if config_path.exists() else {}
args = sys.argv[1:]
with (directory / "repository-requests.jsonl").open("a") as log:
    log.write(json.dumps({"args": args, "cwd": os.getcwd(), "debugPresent": any(key in os.environ for key in ["GH_DEBUG", "GLAB_DEBUG_HTTP"]), "gitlabAuthEnvPresent": any(key in os.environ for key in ["GITLAB_TOKEN", "GITLAB_ACCESS_TOKEN", "OAUTH_TOKEN", "JOB_TOKEN", "CI_JOB_TOKEN", "GLAB_IS_OAUTH2", "OAUTH2_REFRESH_TOKEN", "OAUTH2_EXPIRY_DATE", "REFRESH_TOKEN"])}) + "\n")

def response(body, status=200):
    print(f"HTTP/2.0 {status} Fixture\r\nContent-Type: application/json\r\n\r")
    print(json.dumps(body))
    sys.exit(0 if status == 200 else 1)

if args[:3] != ["api", "--method", "GET"] or "--hostname" not in args or "--include" not in args:
    response({"error": "Only explicit GET fixture requests are supported"}, 500)
host = args[args.index("--hostname") + 1]
endpoint = args[args.index("--include") + 1]
parsed = urlsplit(endpoint)
expected_host = "api.github.com" if host == "github.com" else "gitlab.com"
if parsed.scheme != "https" or parsed.netloc != expected_host or parsed.fragment:
    response({"error": "A fixed absolute public API endpoint is required"}, 500)
endpoint = parsed.path.lstrip("/")
if host == "gitlab.com":
    if not endpoint.startswith("api/v4/"):
        response({"error": "Expected GitLab v4 endpoint"}, 500)
    endpoint = endpoint.removeprefix("api/v4/")
if parsed.query:
    endpoint += "?" + parsed.query
mode = config.get("mode", "success")
if mode == "timeout":
    time.sleep(2)
if mode == "oversized":
    print("x" * (3 * 1024 * 1024))
    sys.exit(0)
if mode == "malformed":
    print("HTTP/2.0 200 OK\r\n\r\nnot JSON")
    sys.exit(0)
if mode == "authentication":
    response({"message": "PRIVATE CREDENTIAL SHOULD NEVER BE RETURNED"}, 401)
if mode == "rate_limit":
    response({"message": "PRIVATE PROVIDER ERROR"}, 429)

revision = "a" * 40
github = host == "github.com"
path = config.get("githubPath", "owner/repo") if github else config.get("gitlabPath", "group/subgroup/repo")
base = "repos/" + path if github else "projects/" + path.replace("/", "%2F")
web = "https://" + host + "/" + path
if endpoint[:len(base)].lower() == base.lower():
    endpoint = base + endpoint[len(base):]

def change(number=7):
    if github:
        return {"number": number, "title": "Tighten the collection", "html_url": web + "/pull/" + str(number), "user": {"login": "fixture"}, "state": "open", "draft": False, "head": {"sha": revision, "ref": "collection"}, "base": {"ref": "main"}, "updated_at": "2026-11-01T12:00:00Z"}
    return {"iid": number, "title": "Tighten the collection", "web_url": web + "/-/merge_requests/" + str(number), "author": {"username": "fixture"}, "state": "opened", "draft": False, "sha": revision, "source_branch": "collection", "target_branch": "main", "updated_at": "2026-11-01T12:00:00Z", "head_pipeline": {"status": "success", "sha": "b" * 40 if mode == "pipeline_revision" else revision, "web_url": web + "/-/pipelines/42"}}

if endpoint == base:
    response({"full_name" if github else "path_with_namespace": path})
kind = "/pulls" if github else "/merge_requests"
if endpoint.startswith(base + kind + "?"):
    count = 20 if mode == "full_page" else 1
    response([change(number) for number in range(7, 7 + count)])
if endpoint == base + kind + "/7":
    item = change()
    if mode == "bad_url":
        item["html_url" if github else "web_url"] = "https://example.com/private"
    if not github and mode == "missing_ci":
        item["head_pipeline"] = None
    response(item)
if github and endpoint == base + "/commits/" + revision + "/check-runs?filter=latest&per_page=100":
    if mode in ["missing_ci", "mixed_ci"]:
        print("PRIVATE CREDENTIAL SHOULD NEVER BE RETURNED", file=sys.stderr)
        response({"message": "PRIVATE PROVIDER ERROR"}, 403)
    items = [] if mode == "no_checks" else [{"name": "Tests", "status": "completed", "conclusion": "future_status" if mode == "unknown_ci" else "success", "head_sha": "b" * 40 if mode == "stale_ci" else revision, "html_url": web + "/actions/runs/42", "details_url": "https://third-party-ci.example/private"}]
    response({"total_count": 101 if mode == "partial_ci" else len(items), "check_runs": items})
if github and endpoint == base + "/commits/" + revision + "/status?per_page=100":
    if mode == "missing_ci":
        response({"message": "PRIVATE PROVIDER ERROR"}, 403)
    items = [{"state": "failure", "context": "Legacy build", "target_url": "https://third-party-ci.example/private"}] if mode in ["failure", "mixed_ci"] else []
    response({"sha": revision, "total_count": len(items), "statuses": items, "state": "failure" if items else "pending"})
response({"error": "Unrecognized fixture GET"}, 404)
