# Software Factory research

Reviewed 31 October 2026. These are architecture inputs and vendor case studies, not independent proof of performance. Recommendations below are our design judgment.

## Requested articles

| Source | Relevant observation | Decision for this project |
| --- | --- | --- |
| [Ramp: Why We Built Our Own Background Agent](https://builders.ramp.com/post/why-we-built-our-background-agent) | Complete isolated environments, verification tools, resumable sessions, shared clients, queued follow-ups, and interruption. | Keep session state outside client lifetimes; attach verification evidence; make queue/interrupt explicit. |
| [Augment: Beyond AI Coding Agents](https://www.augmentcode.com/blog/beyond-ai-coding-agents-how-we-built-augments-software-factory) | Automation spans requirements, tickets, PRs, and production. Review and verification are bottlenecks, and runtime evidence matters. | Treat the factory as delivery workflow; bound work in progress and review queues. |
| [Factory 2.0](https://factory.com/news/software-factory) | Model independence, shared organizational context, production feedback, and graduated autonomy. | Share memory and policy across adapters; increase autonomy through explicit capabilities and observed results. |
| [Firecrawl: How to Build an AI Software Factory](https://www.firecrawl.dev/blog/ai-software-factory) | Intake, isolation, tools, verification, and merge gates. Cheap deterministic checks precede expensive CI and human review. | Build durable intake and verification before scaling worker count. |

All four links were opened through the web tool. Ramp's direct page extraction returned a JavaScript shell; a subsequent official-domain search returned the article content, including its technical sections. The Firecrawl article is a synthesis of other companies' reports; use its linked primary sources before relying on their numerical claims. This project does not repeat their throughput figures as its own expected outcome.

## Provider documentation checked

- [OpenAI: Codex as a platform](https://developers.openai.com/blog/codex-as-a-platform) distinguishes bounded `codex exec` jobs from SDK workflows and app-server product integrations. Use app-server first for Software Factory; other providers begin with their headless CLI adapters.
- [Codex app-server](https://learn.chatgpt.com/docs/app-server) supports thread/turn lifecycle, streaming, native steering, and interruption. `turn/steer` uses an expected active turn identifier. The acknowledgement is not an end-to-end guarantee that a requested change has already been applied.
- [Claude Code programmatic execution](https://code.claude.com/docs/en/headless) supports print mode, streamed JSON, resuming, and structured output. Bare mode does not read subscription OAuth credentials. SIGTERM leaves an unfinished turn; graceful interruption needs different handling.
- [Cursor output format](https://cursor.com/docs/cli/reference/output-format) distinguishes terminal results from activity. Partial streaming includes duplicate buffered/final flush events. A truncated stream must not count as success. [Authentication](https://docs.cursor.com/en/cli/reference/authentication) documents browser login and API keys.
- [Gemini headless reference](https://geminicli.com/docs/cli/headless/) documents streamed JSON events and exit codes. [Authentication](https://geminicli.com/docs/get-started/authentication/) allows reuse of cached authentication; an unattended installation without it needs API-key or Vertex configuration.

Local executable inspection found `codex` and `cursor-agent`/`agent`; it did not find `claude` or `gemini`. A later read-only Codex app-server probe completed initialization and metadata requests. It performed no inference, so credentials, quota, model entitlement, and a successful task remain unverified. See [providers.md](providers.md) and [harness.md](harness.md) for the recorded probe and contract.

## Resulting design

Use a Rust daemon, SQLite, small provider adapters, and shared client state. Preserve the user's objective and steering independently of any provider conversation. Keep the supervisor persistent but event-driven: wake for new evidence or scheduled work, then stop spending tokens when there is no useful action.

The refined UI has one human-facing main agent in Conversation and a separate Progress surface. Graph, board, and factory radar are views of the same work; worker details appear on selection. This keeps steering simple without hiding task-level progress. Visual implementation waits for the user's refined layout choice.

Prefer small bounded runs, isolated worktrees, inspectable progress summaries, and a review bundle tied to a commit. Save verified decisions and lessons with provenance. Do not optimize for raw PR volume, copy entire histories into each prompt, or give a model's self-reported confidence the role of a verifier.

Keep the task board to Backlog, Ready, Running, Review, Done, and Blocked. Verification is activity plus an evidence gate within Running. Run lifecycle uses Queued and Running followed by Succeeded, Failed, Interrupted, Cancelled, or NeedsApproval; timeout is a Failed reason. This separates execution outcomes from accepted delivery.

Start locally with one working provider. Add remote execution when the machine must work while the laptop sleeps; a local background daemon alone cannot satisfy that requirement. Worktrees alone do not provide process or network isolation.

## Experiments to run

These are proposed checks, not reported successful experiments:

| Experiment | Evidence to collect |
| --- | --- |
| Two workers race for one ready task | One active run and one claim event. |
| Daemon stops after a durable claim | Restart exposes interrupted work without duplicate external actions. |
| Provider exits zero without a success result | Run is not promoted to Review. |
| Steering arrives mid-run | Revision persists; acknowledgement and next context are observable. |
| Provider repeatedly fails | Retry limit is honored and blocked reason remains visible. |
| Same automation wake is delivered twice | One occurrence/task is created. |
| Review queue reaches its cap | New implementation dispatch stops; existing evidence remains accessible. |
| Small real provider task | Correct diff, actual verification output, durable event history, clean resumption. |

Compare harness variants using accepted task outcomes, verification completeness, retries, human review time, cost, and elapsed runtime. Keep fixtures deterministic; label real-provider results and environment limits. Propose changes to the harness through the same review process as project code.

Further detail: [architecture.md](architecture.md).
