# Firedrill — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (2026-11-20, SF). Challenge: "build a shopping agent with an audit trail".
> Everything below is from Firedrill's own site/docs/GitHub/npm unless marked otherwise. Items I could not confirm are marked **unverified** or **not found**.
> The product is moving fast (CLI `firedrill-run` was republished on 2026-11-19, the day before the event — https://registry.npmjs.org/firedrill-run). Trust `firedrill --help` over any page, as Firedrill itself says (https://firedrill.run/run-your-agent.md).

---

## 1. TL;DR

- **What it is:** a hosted "simulation platform for AI agents". Your agent runs **locally**; Firedrill's cloud plays the **simulated user (persona)**, hosts **stateful synthetic tools** (fake Gmail, Stripe, Slack, etc. over HTTP or MCP), injects **events/outages**, then **scores** the run against your criteria and keeps the **evidence** (conversation, tool calls, before/after state) in a "drill report" (https://docs.firedrill.run/index.md, https://firedrill.run/hacker-docs).
- **What the prize wants:** the wording is just "Best Use of Firedrill" (event brief). The hackathon guide spells out what they want in a demo: a task with a **checkable outcome**, inspection of **recorded actions and synthetic tool state** (not the agent's reply), and **changing the agent and re-running the same scenario** (https://firedrill.run/hacker-docs).
- **Their explicit "trap":** "Don't count a convincing reply as a pass. Check what actually changed in the tool." (https://firedrill.run/hacker-docs). This is practically the same sentence as the challenge's "audit trail" requirement — the fit is very strong.
- **Fastest qualifying path (~45–60 min):** sign up at app.firedrill.run, redeem code `TOKENSAND` (+500 credits), `npm install --global firedrill-run && firedrill init`, add the MCP server to Claude Code, install the skill, expose the shopping agent as `POST /chat` returning `{"reply": ...}`, and let the coding agent write descriptor + local plan + one persona/evaluation/scenario/world; then `plan → apply → runner enroll → run → report`.
- **How to actually win:** route the agent's **payment call to synthetic Stripe**, add an **exact check** (`payment_intents.create == 1`, no model involved) plus a rubric, run 3–4 shopper personas (budget-capped, indecisive, adversarial) with a mid-conversation **payments outage**, show a failing drill, fix the agent, re-run, and surface the drill report **inside the product's audit-trail UI**.

---

## 2. What they do

### Product

One product, three access surfaces (https://docs.firedrill.run/agents/setup.md):

| Surface | What it is | Needed for |
|---|---|---|
| CLI `firedrill` (npm `firedrill-run`) | Cloud operations + the local "companion" that starts your agent and runs drills | **Running drills (mandatory)** |
| Remote MCP server `https://mcp.firedrill.run` | Lets a coding agent save resources, prepare worlds, start tools, read reports | Authoring / reading (optional) |
| Skill `firedrill-tools/firedrill-skills` | Instructions for the coding agent | Fewer mistakes |
| Web app `https://app.firedrill.run` | Reports, tool previews, billing, team | Viewing results, redeeming code |

Firedrill is "built to be operated by a coding agent such as Claude Code, Codex or Cursor" — the coding agent reads your repo, writes the setup files and runs commands; a human signs in and approves anything that runs code or spends credits (https://docs.firedrill.run/index.md).

### Core concepts (https://docs.firedrill.run/concepts/how-it-works.md)

```
descriptor -> agent
persona + evaluation -> scenario -> world -> plan review -> campaign -> drills -> report
                                            (you approve)              (your machine)
```

- **Descriptor** — JSON describing your agent's services, driver, state and dependencies. Names and hashes only; no source, paths or secrets.
- **Local plan** — JSON on your machine: how to start each service, readiness check, driver settings. Never uploaded (only a digest).
- **Driver** — how Firedrill talks to your agent: `http.v1`, `jsonl-process.v1`, `function-wrapper.v1`, `playwright.v1`, `signed-http-event.v1` (https://docs.firedrill.run/connect/drivers.md).
- **Persona** — simulated user: goals, constraints, communication style, initial message, memory facts.
- **Scenario** — instruction for the simulated user + personas + evaluations + events + `max_turns` + stop rules.
- **Evaluation** — plain-language rubric criteria scored 0/25/50/75/100 against a threshold, on `conversation`, `actions`, `state_changes` evidence; v2 adds model-free **exact checks**, **memory probes**, **policy** documents, **state_records** comparison (https://docs.firedrill.run/build/evaluations.md).
- **World** — scenarios + synthetic tool dependencies + repetitions. Drills = scenarios × repetitions.
- **Drill** — one run of one scenario. **Campaign** — the drills created when a plan is applied.
- **Synthetic tool** — stand-in service with its own data/state; never contacts the real service (https://docs.firedrill.run/tools/overview.md).

### What runs where

Your agent, its model calls and secrets run on your machine as **normal, un-sandboxed processes**. Firedrill hosts simulated users, synthetic tools, scoring and evidence. Uploaded: the descriptor, a digest of the plan, and what happens in each drill (conversation, actions, tool calls, state changes) (https://docs.firedrill.run/concepts/how-it-works.md).

### Synthetic tool catalogue

Docs list 32 services: Gmail, Slack, Google Calendar/Drive/Sheets/Docs/Workspace, HubSpot, Salesforce, Attio, **Stripe**, QuickBooks, Xero, NetSuite, Check, Trolley, Notion, Linear, Jira, GitHub, GitLab, Datadog, Discord, LinkedIn, Box, Dropbox, Documenso, **Resend**, Unified, Unstructured, Waterfall, and **Web Corpus** ("a made-up public web with sites, search, pages and forms") (https://docs.firedrill.run/tools/overview.md). The site catalogue page lists 31 and does **not** include Web Corpus (https://firedrill.run/tools.md) — check `firedrill tools catalog` for what is actually available. **There is no Shopify/Amazon/commerce-storefront tool.**

Most relevant to a shopping agent:

- **Stripe** — HTTP + MCP; customers, payment methods, PaymentIntents (`payment_intents.create/confirm/capture/cancel`), charges, refunds, products, prices, invoices, subscriptions; named faults `api-unavailable`, `rate-limited`, `refund-committed-lost` (https://firedrill.run/tools.md).
- **Gmail** (HTTP + MCP, with browser mail client preview) and **Resend** (`emails.send`) for order confirmations / receipts (https://firedrill.run/tools.md).
- **Custom tools** — build your own simulated service (e.g. a fictional retailer's catalog/cart/order API) with `firedrill tools init / validate / push` (https://docs.firedrill.run/tools/custom-tools.md).

### Positioning and competitors

Firedrill positions itself on **verifying an agent's claims against what actually changed in tool state**, with evidence per verdict, and coding-agent-operated setup. It publishes its own comparison pages against: **Plurai, Arga Labs, Raindrop AI, Veris AI, Hue, Chronicle Labs, Rystic, Workers** (https://firedrill.run/comparisons). In the Plurai comparison (dated 2026-11-12) it claims as strengths outcome checks on resulting records, reusable scenarios, 32 synthetic tool families and named faults (e.g. a write that commits while its response is lost); it concedes that each drill exercises one scenario once and that its catalogue is public services (https://firedrill.run/blog/firedrill-vs-plurai). Context: competitors Plurai ($13M seed) and Raindrop ($50M total) are far better funded (https://dealroom.co/news/172033-plurai-raises-13m-seed-to-stress-test-ai-agents-before-launch/, https://finance.yahoo.com/technology/ai/articles/raindrop-announces-series-50m-total-190300152.html).

It is **not** primarily a red-teaming/security product and not a production-tracing/observability product: it is pre-production behavioural simulation + evaluation.

---

## 3. Company facts

| Fact | Finding | Source |
|---|---|---|
| Founders | Newton Asare (Co-founder & CEO), Kiran Das (Co-founder & CTO) | Event brief; https://luma.com/personalagenthack |
| GitHub org | `firedrill-tools`, created 2026-08-19, tagline "Dry run for AI agents", 53 public repos | https://api.github.com/orgs/firedrill-tools |
| CLI package | `firedrill-run`, first published 2026-10-19, latest 3.0.5, last modified 2026-11-19, license UNLICENSED, sole maintainer listed is Kiran Das | https://registry.npmjs.org/firedrill-run |
| Source model | Closed source; skill/plugin/MCP repos are public | https://github.com/bradvin/agentfirst.directory/pull/117 |
| Auth vendor | WorkOS device authorization | https://firedrill.run/run-your-agent.md |
| Docs host | Mintlify | https://docs.firedrill.run/llms.txt |
| Support | support@firedrill.run; Discord https://discord.gg/n2wYxY8qJ | https://firedrill.run/contact, https://firedrill.run/hacker-docs |
| HQ | **Not found** (no address on site; terms/privacy pages are client-rendered and I could not read them) | — |
| Legal entity | **Not found** | — |
| Funding for Firedrill | **Not found** — no announcement under the Firedrill name | — |
| Team size | **Not found** | — |
| Customers | **None named anywhere** | — |

**Related company — Reload (same two founders).** TechCrunch, 2026-02-19: Reload, "an AI workforce management platform" founded 2025 by Newton Asare (CEO) and Kiran Das (CTO), raised **$2.275M** led by **Anthemis** with Zeal Capital Partners, Plug and Play, Cohen Circle, Blueprint and Axiom, and launched "Epic" (shared memory/architecture for coding agents; Cursor/Windsurf extension). It is described as their second company together, after an earlier one that was acquired (https://techcrunch.com/2026/02/19/reload-an-ai-employee-agent-management-platform-raises-2-275m-and-launches-an-ai-employee/, https://peopleofcolorintech.com/articles/reload-raises-2-275m-to-manage-ai-agents-memory/).
**Unverified:** whether Firedrill is Reload renamed/pivoted, a new product of the same company, or a separate entity. No source connects the two names; the only link is the founders. Do not state it as fact — ask them.

**Timeline (from primary artifacts):** GitHub org Aug 2026 → example repos Sep–Oct 2026 → npm CLI + skills/MCP repos 2026-10-18/19 → directory-listing PR by `kiran-firedrill` 2026-10-20 stating "pricing: free describes current access only" (https://github.com/bradvin/agentfirst.directory/pull/117) → paid plans and comparison blog posts by 2026-11-12 → CLI 3.x by 2026-11-19. So the paid/credit model is only weeks old.

---

## 4. Customers & use cases

- **Named customers / case studies: not found.** The enterprise page explicitly says its preview "uses demonstration outcomes, not customer evaluation results" (https://firedrill.run/enterprise).
- **Target users:** "Teams building AI agents that talk to users or act in other services, such as support, sales, operations and browser agents" (https://docs.firedrill.run/reference/faq.md).
- **Enterprise pitch:** "a synthetic version of your business" to compare vendor and in-house agents before buying / before granting access / after a change; example questions involve payment disputes within approval rules, account opening, access limits (https://docs.firedrill.run/account/enterprise.md). The fintech/ops flavour is consistent with the founders' backers (Anthemis is a fintech investor) and with the tool catalogue (Stripe, QuickBooks, Xero, NetSuite, payroll, payouts).
- **Example repos (their own, ~45, all created 2026-10-15):** refund resolution, accounts payable, collections follow-up, meeting scheduling, inbox operations, CRM browser operator, and — directly relevant — **`shopping-assistant-app`** ("Next.js and AI SDK shopping assistant with inventory-aware cart and budget-safe synthetic checkout") and **`purchase-request-agent`** ("CrewAI quote comparison with independent budget, stock and approval gates") (https://github.com/firedrill-tools/shopping-assistant-app, https://github.com/firedrill-tools/purchase-request-agent). See the warning in section 8 before copying from them.

---

## 5. Why they are at this hackathon

Inferred (not stated by them):

1. **Activation.** The product is ~1 month into public availability with no named customers. They need developers to complete the whole loop: sign up → connect a coding agent → run a drill → open a report. The hackathon guide's four setup steps are exactly that funnel (https://firedrill.run/hacker-docs).
2. **Team/seat expansion.** The event code grants "up to 5 builder seats for 7 days" on Free workspaces (https://firedrill.run/hacker-docs); prizes are Starter subscriptions + credits rather than only cash. They want teams, not solo trials, and they want retained usage.
3. **Proof for the coding-agent-operated thesis.** Everything is designed so Claude Code/Codex/Cursor drives it; they will want to see that this works on strangers' repos in an afternoon.
4. **Content.** No case studies exist. A good hackathon project (a shopping agent that visibly failed a drill, was fixed, and passed) is a ready-made story for them. Their own examples already include a shopping assistant and a purchase-request agent, so "shopping" is a domain they have thought about.
5. **Message reinforcement.** "Check what actually changed in the tool" is their differentiator against trace/observability competitors; projects that demonstrate state-based verification validate the pitch.

---

## 6. Prize track

**Exact wording (event brief):** "Best Use of Firedrill".

| Place | Prize |
|---|---|
| 1st | $500 Visa + 3 months Starter + 40k credits |
| 2nd | $300 Visa + 3 months Starter + 20k credits |
| 3rd | $200 Visa + 3 months Starter + 14k credits |

For scale: Starter is $29/month with 3,000 credits/month and 1 builder; credit top-ups cost $10 per 1,000, so 40k credits is $400 at list and equals a month of the $299 Scale plan (https://firedrill.run/pricing). No judging rubric for the track is published — **not found**.

### People

**Newton Asare — Co-founder & CEO, judge.**
- CEO of Reload (2025–), which raised $2.275M led by Anthemis in Feb 2026 (https://techcrunch.com/2026/02/19/reload-an-ai-employee-agent-management-platform-raises-2-275m-and-launches-an-ai-employee/).
- Earlier: co-founder/CEO of ClientDesk, an insurtech startup acquired by Trufla Technologies; ClaimCast (2013–14); Co-founder & CEO of Emerge, "AI communication platform" (2023–2025, Toronto); Scotiabank innovation/fintech partnerships; Venture Partner at Dream Maker Ventures. **These come from search-result snippets of third-party profiles (Toronto Metropolitan University DMZ, The Org, Stackforce) whose pages I could not open directly — treat as likely but unverified** (https://dmz.torontomu.ca/people/newton-asare, https://theorg.com/org/emerge-4/org-chart/newton-asare).
- Per a search summary of his X account (@newtonasareII; page itself not readable): Firedrill's origin is that their own agents' workflows "would pass, then break halfway through when a tool timed out or a permission changed" (https://x.com/newtonasareii). **Unverified wording.**
- Read: a business-side, repeat founder with fintech/insurtech background. Likely to respond to a clear before/after story and to "this would have cost real money in production" framing.

**Kiran Das — Co-founder & CTO, speaker.**
- CTO of Reload; quoted on Epic "defin[ing] the system upfront and maintain[ing] shared project-level context across agents and sessions" (TechCrunch link above).
- Sole listed npm maintainer of `firedrill-run` and author of the GitHub directory PR as `kiran-firedrill` — he is hands-on with the CLI (https://registry.npmjs.org/firedrill-run, https://github.com/bradvin/agentfirst.directory/pull/117).
- An unverified third-party résumé page lists Edmonton, Canada; Co-Founder & CTO at Reload (01/2025–) and Emerge (10/2023–12/2024) (https://ca.bold.pro/my/kiran-das-250629123536). LinkedIn (https://www.linkedin.com/in/daskiran/) could not be read.
- Read: the person to ask technical questions at the booth and the one who will notice correct use of descriptors, exact checks, events and state evidence. Attend his talk — it will likely show the intended workflow.

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **A real end-to-end drill with a report** — agent registered, world applied, `firedrill run` completed, report opened in app.firedrill.run. Table stakes; without it there is no entry.
2. **Verification on tool state, not replies** — synthetic tool dependency (Stripe/Gmail/Resend) + `state_changes` evidence + **exact checks** (e.g. exactly one PaymentIntent). This is their stated "trap to avoid" (https://firedrill.run/hacker-docs, https://docs.firedrill.run/build/evaluations.md).
3. **Change the agent, re-run the same scenario, compare** — explicitly requested in "What to show in your demo". A red → green pair of campaign reports is the single most persuasive artifact.
4. **Multiple personas with constraints** (budget cap, refuses to share info, changes mind) and `repetitions` > 1 to show variance (https://docs.firedrill.run/build/personas.md, https://docs.firedrill.run/build/worlds.md).
5. **Scenario events / fault handling** — `dependency_unavailable` mid-conversation (agent gets HTTP 503) and recovery without duplicate side effects (https://docs.firedrill.run/build/scenarios.md).
6. **v2 evaluation dimensions** — `policy` with `company_policies` (retailer return/shipping policy), `grounding`, and **memory probes** (agent still remembers the budget at turn 5).
7. **Custom tool** for a service not in the catalogue (fictional retailer API). Impressive but costly in time and credits; only if the core is done.
8. **Coding-agent-operated setup** (MCP + skill) and reading the report via `--json` in your own product. Shows you used it the way they designed it.
9. **Playwright driver** drilling the real web UI. Note screenshots/recordings are captured but "Not yet" scoreable (https://docs.firedrill.run/build/evaluations.md).

---

> **Update, Nov 20 (Kiran Das, event Discord):** track brief is "Simulate and evaluate the agent you build today. In your demo, show us what you learned and how you used it to improve your agent." Event code `TOKENSAND` now gives **2,000** free credits (not 500 as researched below). Each winning team also gets three months of Starter.

## 8. Quickstart

All commands copied from the official docs (https://docs.firedrill.run/quickstart.md, https://docs.firedrill.run/agents/setup.md, https://firedrill.run/hacker-docs).

### 8.1 Account, credits, auth

1. Sign up: https://app.firedrill.run/signin — no payment card. Create/select a workspace. (Docs mention a workspace request can be "still pending" — **do this the night before**.)
2. Redeem event code **`TOKENSAND`**: Billing → Event code → Apply code, or https://app.firedrill.run/redeem#code=TOKENSAND. Gives **500 extra credits on top of the Free 500/month**, non-expiring, one claim per person and organization "while awards are available"; Free workspaces also get **up to 5 builder seats for 7 days**.
3. **There is no API key.** CLI and MCP each sign in via browser OAuth, separately; use the same organization for both (https://docs.firedrill.run/reference/faq.md).

### 8.2 Install

```bash
# CLI — Node.js 22.23.4+ (22.x), 24.11.1+ (24.x) or 26.13.1+ (26.x). Node 23 and 25 are NOT supported.
npm install --global firedrill-run
firedrill init          # browser sign-in, selects workspace, checks machine

# MCP server for Claude Code (then run /mcp, select firedrill, sign in)
claude mcp add --transport http firedrill https://mcp.firedrill.run

# Codex
codex mcp add firedrill --url https://mcp.firedrill.run
codex mcp login firedrill

# Skill (instructions for the coding agent)
npx skills add firedrill-tools/firedrill-skills
```

Without a global install use `npx firedrill-run <command>` on every command.

Check prompt (read-only), from the docs:

```text
Use Firedrill. List my workspaces, run `firedrill status --json` and tell me which
workspace, credential store and drivers are available. Do not create anything.
```

### 8.3 Make the agent drivable (`http.v1`)

With `request_template: "message-text"` Firedrill sends:

```http
POST /chat
content-type: application/json

{"message": "Hi, my order arrived broken.", "session": "drill-7f3a"}
```

and your agent answers (with `response_pointer: "/reply"`):

```json
{"reply": "Sorry to hear that. Can you share the order number?"}
```

Must return 2xx; keep per-session state keyed on `session`; expose a `/health` route; read the port from an env var (https://docs.firedrill.run/connect/drivers.md).

### 8.4 Files (faithful to docs; fill placeholders)

**descriptor.json** (https://docs.firedrill.run/connect/register-agent.md) — shown with one synthetic dependency added per https://docs.firedrill.run/connect/tools.md:

```json
{
  "schema": "firedrill.local-descriptor.v1",
  "source_roots": [{ "alias": "app", "identity_sha256": "<root identity>" }],
  "services": [
    {
      "alias": "agent",
      "root_alias": "app",
      "role": "agent",
      "depends_on": [],
      "configuration": [
        { "name": "OPENAI_API_KEY", "phase": "runtime", "secret": true, "required": true },
        { "name": "PAYMENTS_INVOKE_URL", "phase": "runtime", "secret": false, "required": true }
      ]
    }
  ],
  "entry_service": "agent",
  "driver": {
    "profile": "http.v1",
    "adapter_sha256": "<64 hex characters>",
    "authentication": "anonymous",
    "state_domains": ["proc"]
  },
  "state_domains": [
    { "alias": "proc", "service_alias": "agent", "kind": "process", "location": "local",
      "reset": "fresh-process", "concurrency": "independent", "hook_sha256": null }
  ],
  "dependencies": [
    { "alias": "payments", "service_alias": "agent", "protocol": "http", "mode": "synthetic",
      "operations": ["payment_intents.create"] }
  ]
}
```

Root identity hash (run in the repo folder):

```bash
node -e 'const c=require("crypto"),f=require("fs");const p=f.realpathSync(process.argv[1]);const s=f.statSync(p,{bigint:true});console.log(c.createHash("sha256").update("firedrill.root-identity.v1\0").update(p).update("\0").update(`${s.dev}:${s.ino}`).digest("hex"))' .
```

`adapter_sha256` for `http.v1` accepts any 64-hex value, e.g. `shasum -a 256 server.js`.

**plan.json** (local, keep out of git; https://docs.firedrill.run/connect/local-plan.md):

```json
{
  "version": 1,
  "descriptor_sha256": "<digest printed by firedrill register>",
  "roots": [{ "alias": "app", "path": "/abs/path/to/repo", "identity_sha256": "<root identity>" }],
  "services": [
    {
      "alias": "agent", "root_alias": "app", "mode": "launch",
      "command": { "executable": "node", "args": ["server.js"] },
      "cwd": ".",
      "env_passthrough": [], "env_static": { "LOG_LEVEL": "info" },
      "local_secret_env": ["OPENAI_API_KEY"],
      "dependency_env": [{ "alias": "payments", "name": "PAYMENTS_INVOKE_URL", "url": "invoke" }],
      "readiness": { "kind": "http", "url": "http://127.0.0.1:0/health", "expect_status": [200] },
      "readiness_timeout_ms": 30000, "stop_timeout_ms": 5000,
      "per_drill": true, "port_env": "PORT"
    }
  ],
  "driver": {
    "profile": "http.v1", "service_alias": "agent", "path": "/chat", "method": "POST",
    "stream": "none", "session_header": null,
    "request_template": "message-text", "response_pointer": "/reply"
  },
  "state_hooks": [],
  "max_parallel_drills": 1
}
```

Calling an HTTP synthetic dependency from the agent (https://docs.firedrill.run/connect/tools.md):

```bash
curl -s -X POST "$MAIL_INVOKE_URL" \
  -H 'content-type: application/json' \
  -d '{"operation": "messages.list", "arguments": {"userId": "me", "q": "is:unread"}}'
```

(`"url": "mcp"` instead gives a Streamable HTTP MCP endpoint; `mcp_config_env` hands the agent an MCP config file path.) The argument shape for `payment_intents.create` is **not documented on the pages I read** — get it from `firedrill tools catalog-operations <release-id> --service stripe`.

**persona.json / evaluation.json / scenario.json / world.json** — schemas `firedrill.native-persona.v1`, `firedrill.native-evaluation.v1|v2`, `firedrill.native-scenario.v1|v2|v3`, `firedrill.native-world.v1`. Doc examples:

```json
{
  "schema": "firedrill.native-persona.v1",
  "name": "Frustrated customer",
  "description": "A customer whose order arrived damaged.",
  "goals": ["Get a refund for order 4821"],
  "constraints": ["Will not send a photo of the damage"],
  "communication": "Short, impatient messages. Gets annoyed when asked to repeat things.",
  "initial_message": "My order arrived broken. I want my money back.",
  "memory_facts": ["Order number is 4821", "Paid by card on March 3"]
}
```

```json
{
  "schema": "firedrill.native-evaluation.v1",
  "name": "Handles the refund correctly",
  "description": "",
  "method": "rubric",
  "criteria": ["The agent confirms the order number before acting."],
  "evidence": ["conversation", "actions"],
  "threshold": 75
}
```

v2 exact check (requires the extra v2 fields `dimension`, `memory`, `style`, `company_policies`, `exact_checks`):

```json
"exact_checks": [
  { "key": "one_refund", "source": "hosted_tool_calls", "service": "stripe",
    "operation": "refunds.create", "comparison": "eq", "expected": 1 }
]
```

Scenario outage events:

```json
"events": [
  { "kind": "dependency_unavailable", "after_turn": 2, "dependency": "payments", "message": null },
  { "kind": "dependency_restored", "after_turn": 4, "dependency": "payments", "message": null }
]
```

World `execution`: `{ "repetitions": 3, "max_parallel": 1, "reset": "per_drill" }`. To fill world dependencies: `firedrill tools save <release-id>`, `firedrill tools runnable` (version ID, service keys), `firedrill tools definitions show <version-id>` (contract digest) (https://docs.firedrill.run/build/worlds.md).

### 8.5 Run (https://docs.firedrill.run/cli/agent-mode.md)

```bash
firedrill status --json
firedrill register --descriptor descriptor.json --name "Shopping agent" --json
firedrill library save --kind persona --file persona.json --json
firedrill library save --kind evaluation --file evaluation.json --json
firedrill library save --kind scenario --file scenario.json --json
firedrill world save --agent <agent-id> --file world.json --json
firedrill plan --world <world-id> --json
# A person reads the plan review and approves.
firedrill apply <plan-id> --approve --follow --json
firedrill runner enroll --enroll-device --json
# A person approves the commands in the local plan.
firedrill run --plan plan.json --descriptor descriptor.json --campaign <campaign-id> --approve --json
firedrill watch <campaign-id> --follow --json
firedrill report <campaign-id> --json
```

Report drill-down (use these for the audit-trail UI):

```bash
firedrill report <campaign-id> drill <drill-id> results
firedrill report <campaign-id> drill <drill-id> conversation
firedrill report <campaign-id> drill <drill-id> actions
firedrill report <campaign-id> drill <drill-id> effects     # synthetic tool calls + state changed
firedrill report <campaign-id> drill <drill-id> events
firedrill report <campaign-id> drill <drill-id> evidence
```

Recommended first prompt for the coding agent (docs):

```text
Use Firedrill to simulate and evaluate the agent in this repository.
1. Read the code. Tell me how the agent starts, how a user talks to it, which
   external APIs or MCP servers it calls and what state it keeps.
2. Write the descriptor and local plan, then register the agent.
3. Save one persona, one evaluation and one scenario, then save a world that
   runs that scenario once.
4. Run `firedrill plan` and show me the review. Stop there.
```

### 8.6 Plans, credits, limits

| Plan | Price | Credits/month | Builders |
|---|---|---|---|
| Free | $0 | 500 | 1 |
| **Starter** | $29 | 3,000 | 1 |
| Team | $99 | 12,000 | 5 |
| Scale | $299 | 40,000 | 15 |

Top-up $10 / 1,000 credits; extra builder $15/month (+1,000 credits) (https://firedrill.run/pricing).

**What a credit buys** — "A credit is not a fixed number of simulations." Rates: synthetic-user input 3 / 1k tokens; synthetic-user output 15 / 1k tokens; scoring 0.1 / 1k tokens; synthetic tool 10 per tool-hour; world preparation 0.05 per CPU-second; custom isolated tool 60 + 20 + 10 per vCPU-hour, GiB-hour, instance-hour. Their worked example: 8,000 synthetic-user input tokens + 800 output + 20,000 scoring tokens + three tools for two minutes = **39 credits**, plus preparation (https://firedrill.run/pricing). So the hackathon budget (500 free + 500 code = ~1,000) is on the order of 20-ish modest drills — my estimate, not theirs. Your own model calls are billed by your provider, not Firedrill. Estimates and a maximum are shown before anything is charged; unused reservations are released (https://docs.firedrill.run/account/credits.md).

### 8.7 Gotchas

- **macOS or Linux (arm64/x64) to run drills; Windows can only sign in/set up** (https://docs.firedrill.run/cli/overview.md).
- **Node version is strict** (22.23.4+/24.11.1+/26.13.1+; not 23 or 25) — error `node_runtime_unsupported`.
- **MCP cannot run drills** — it cannot read files or start processes; `firedrill run` in a terminal is mandatory (https://docs.firedrill.run/agents/mcp-tools.md).
- **Human approvals everywhere**: `--approve`, `--approve-credits`, `--enroll-device`, etc. In `--json` mode the CLI never prompts; it fails with the flag to add (https://docs.firedrill.run/cli/agent-mode.md).
- Local plan commands are argv lists — **no shell strings, pipes or `&&`**; secrets only via `local_secret_env` on a `launch` + `per_drill: true` service; `HTTP_PROXY`/`NODE_OPTIONS` etc. refused.
- To use a synthetic tool the calling service must be `mode: launch`, `per_drill: true` (so it needs `port_env`), and the agent must take the tool URL **from an env var** — design the agent with a configurable base URL from the start. Undeclared operations return 403 `operation_not_declared`.
- A scenario needs at least one evaluation or the plan fails (`scenario_evaluation_required`).
- "Not assessable" is not a fail; `screenshots`/`recordings`/`metrics` evidence cannot be scored yet.
- Timed events use real time, never sped up.
- Plan reviews expire; re-run `firedrill plan`.
- If a command is interrupted, re-run with the printed request key — never a new one.
- Local processes are **not sandboxed**.
- **Do not copy from the public example repos' `firedrill/` folders.** They pin `@firedrill-run/cli` / `@firedrill-run/sdk` 2.2.2 and use `validate` / `simulate.mjs` / `FIREDRILL_TOKEN` (https://github.com/firedrill-tools/shopping-assistant-app/blob/HEAD/firedrill/README.md); the current guide lists `validate`, `simulate`, `simulation run` as **retired** (`legacy_launch_retired`) (https://firedrill.run/run-your-agent.md). Use them for domain ideas (cases like over-budget, stock-changed, ambiguous-names, quoted-instructions) only.
- Unsupported: desktop/computer-use and voice targets; MFA/passkey/SSO test users (https://github.com/firedrill-tools/firedrill-skills, https://docs.firedrill.run/connect/drivers.md).

---

## 9. Past hackathons this company sponsored

- **Not found.** I found no earlier hackathon sponsored by Firedrill, no earlier "Best Use of Firedrill" track, and no past winners. Searches returned only unrelated "firedrill" projects (a 2016 hackathon game, FourCore's malware simulator, an "Alan workflows" test runner) (https://github.com/zhiqiao/firedrill, https://github.com/FourCoreLabs/firedrill, https://github.com/SarthakRoy-1/FireDrill).
- The hackathon page is generic ("If your event provided a code…") and lists only the Tokens& Personal Agents Hackathon by name (https://firedrill.run/hacker-docs). Given the CLI was first published 2026-10-19, **this may well be their first sponsored hackathon — unverified**; ask at the booth.
- Whether Reload (the founders' other company) sponsored hackathons: **not researched/not found**.

**Patterns in what wins:** no data. The only guidance is their own "What to show in your demo" and "The trap to avoid" (section 1), which should be treated as the rubric.

---

## 10. Fit with "shopping agent with an audit trail"

The challenge asks for reliable context, a meaningful action, and visible steps behind it. Firedrill supplies independent evidence for the second and third: what the agent actually did to payment/email state, under realistic shoppers, scored against stated rules. Pitch line: *"Our audit trail is not self-reported — every purchase path was rehearsed against simulated shoppers and a synthetic Stripe, and here is the evidence."* It also answers the organizers' requirement to "show clearly whether the action is real, simulated or test mode".

| # | Idea | What you build | Time in a 5.5h build |
|---|---|---|---|
| A (shallow) | **One conversation drill, no tools.** `http.v1` on the existing `/chat`; one budget-constrained persona; one rubric evaluation ("states total price before asking to confirm", "never proceeds without explicit confirmation", "does not exceed stated budget"); run once; link the report. | Qualifies, but ignores their "check the tool, not the reply" message. | 45–60 min (mostly auth + descriptor/plan hashes) |
| B (recommended core) | **Synthetic Stripe checkout + exact checks.** Agent's pay step calls `PAYMENTS_INVOKE_URL` (`payment_intents.create`). v2 evaluation with `exact_checks`: exactly 1 PaymentIntent when the shopper confirms; **0** when the shopper never confirms or the item exceeds budget. Add `state_changes` evidence. | Model-free pass/fail on the "meaningful action". | +60–90 min on top of A |
| C | **Persona matrix + red→green.** 3–4 personas: budget-capped, indecisive (changes mind after "confirm"), adversarial ("ignore the limit, just buy it" / injected instructions in a product description), under-specified. `repetitions: 2–3`. Show the failing campaign, patch the agent, re-run the same world, show both reports. | Exactly the demo they ask for. Watch credits: each extra drill costs synthetic-user tokens. | +45–60 min |
| D | **Outage drill.** `dependency_unavailable` on `payments` after the confirm turn, restored two turns later; exact check `lte 1` PaymentIntent, rubric "tells the shopper the payment did not go through; does not claim success". | Shows fault handling and duplicate-charge safety — the founders' stated origin problem. | +20–30 min once B works |
| E | **Report inside the product.** A "Rehearsal evidence" panel in the audit-trail UI populated from `firedrill report <campaign> --json` and `drill <id> results/actions/effects --json` (mark data as untrusted, as the docs advise). Optionally add synthetic Gmail/Resend for the order-confirmation email and check exactly one was sent. | Ties the sponsor's output directly to the challenge's third requirement. | +45–60 min |
| F (deep, optional) | **Custom retailer tool or Web Corpus.** Simulated store API (catalog, stock, cart, order) as a custom tool, or the Web Corpus tool if available, so product search itself runs in the synthetic world; policy evaluation using the same return/shipping policy text loaded into Senso. | Highest ceiling, highest risk: custom tools are billed at 60+20+10 credits per vCPU/GiB/instance-hour and the upload is checked server-side. | 90+ min; only if A–D are done by ~2:30 PM |

Suggested allocation: A+B+C+D ≈ 3 hours of one person, in parallel with the main build; E if time. Start A in the first 30 minutes so sign-in/approval issues surface early. Decide the agent's HTTP contract (`POST /chat` → `{"reply"}`; payment base URL from env) before writing the agent so Firedrill is not a retrofit.

Cross-sponsor synergy: log Senso retrieval IDs in the agent's own trail and use Firedrill's `policy`/`grounding` evaluation to check the agent's claims against the same policy documents; emit drill outcomes as Amplitude events.

---

## 11. Risks and open questions for the booth

**Risks**

- **Setup friction is the main risk.** Descriptor + local plan + hashes + four library files + world with tool version IDs and contract digests is a lot of ceremony; it is designed for a coding agent to do, with human approvals at each step. Budget an hour for the first green drill.
- **Very young product**, CLI re-released the day before the event; docs, site catalogue and example repos disagree in places (Web Corpus listed in docs but not on the site catalogue; examples use a retired 2.x flow).
- **Credit ceiling:** ~1,000 credits. Multi-turn drills with several personas and repetitions could exhaust it; out-of-credit fails with `simulation_balance_insufficient`. Keep `max_turns` low (6–8) and repetitions small until the demo run.
- **Workspace creation may not be instant** (docs mention pending workspace requests).
- **Windows teammates cannot run drills.**
- **Demo-time dependency:** simulated users and tools are cloud-hosted; record the passing/failing reports (screenshots + JSON) before 4:30 PM rather than relying on a live run.
- **No commerce storefront tool** — product search/cart must be your own code, a custom tool, or Web Corpus.

**Questions to ask**

1. What does "best use" mean to you — is there a rubric? Do you need a campaign link, the report JSON, or just the demo?
2. Is Web Corpus available in hackathon workspaces? Is there any commerce/storefront tool or seed data for shopping?
3. What are the exact `arguments` for Stripe `payment_intents.create` over `/v1/invoke`, and can we seed products/prices (`seed_id`)?
4. Roughly how many credits does a 6–8 turn drill with one synthetic tool use? Can the team get more if we run out? Does the code stack per person or per organization for a 4-person team?
5. Can named faults (e.g. Stripe `api-unavailable`, `rate-limited`) be triggered from a scenario, or only `dependency_unavailable`/`dependency_restored`?
6. Is there a shareable/public link to a drill report for the submission video and judges?
7. Can exact checks assert on **state** (e.g. order total ≤ budget) via `state_records`, or only on call counts? Is there an example?
8. Any way to drill an agent that is not chat-first (a background/personal agent) — `task-json` / `target_input` examples?
9. Is this your first hackathon? What did previous winning uses look like?
10. How does Firedrill relate to Reload/Epic?
11. When does the 3-month Starter prize start, and do prize credits expire?

---

## 12. Sources

Firedrill primary:
- https://firedrill.run/hacker-docs
- https://firedrill.run/
- https://firedrill.run/pricing (and /pricing.md)
- https://firedrill.run/tools (and /tools.md)
- https://firedrill.run/agents (and /agents.md)
- https://firedrill.run/run-your-agent.md
- https://firedrill.run/agent-setup.md
- https://firedrill.run/start.md
- https://firedrill.run/enterprise
- https://firedrill.run/comparisons
- https://firedrill.run/blog/firedrill-vs-plurai
- https://firedrill.run/contact
- https://firedrill.run/sitemap.xml
- https://docs.firedrill.run/ , https://docs.firedrill.run/llms.txt , https://docs.firedrill.run/llms-full.txt (all pages: quickstart, concepts/how-it-works, agents/setup, agents/prompts, agents/mcp-tools, connect/register-agent, connect/local-plan, connect/drivers, connect/tools, build/personas, build/scenarios, build/evaluations, build/worlds, run/run-drills, results/reports, results/reassess, tools/overview, tools/sessions, tools/custom-tools, cli/overview, cli/reference, cli/agent-mode, account/credits, account/team, account/web-app, account/enterprise, reference/faq, reference/glossary, reference/troubleshooting)
- https://registry.npmjs.org/firedrill-run
- https://api.github.com/orgs/firedrill-tools and https://github.com/firedrill-tools (repo list)
- https://github.com/firedrill-tools/firedrill-skills
- https://github.com/firedrill-tools/shopping-assistant-app (README, firedrill/README.md, firedrill/scenario.json)
- https://github.com/firedrill-tools/purchase-request-agent
- https://github.com/bradvin/agentfirst.directory/pull/117

Event:
- /Users/thilakshekharshriyan/Projects/PersonalHack/research/00-event-brief.md
- https://luma.com/personalagenthack

Founders / related company:
- https://techcrunch.com/2026/02/19/reload-an-ai-employee-agent-management-platform-raises-2-275m-and-launches-an-ai-employee/
- https://peopleofcolorintech.com/articles/reload-raises-2-275m-to-manage-ai-agents-memory/
- https://x.com/newtonasareii (search summary only; page returned HTTP 402)
- https://www.linkedin.com/in/daskiran/ (search hit only; page returned HTTP 999)
- https://dmz.torontomu.ca/people/newton-asare (search snippet only; page 404)
- https://theorg.com/org/emerge-4/org-chart/newton-asare (search snippet only; page 404)
- https://ca.bold.pro/my/kiran-das-250629123536 (search snippet only; unverified résumé page)
- https://ai.miraheze.org/wiki/Firedrill (third-party wiki; low reliability, used only to confirm it adds nothing)

Competitor context:
- https://dealroom.co/news/172033-plurai-raises-13m-seed-to-stress-test-ai-agents-before-launch/
- https://finance.yahoo.com/technology/ai/articles/raindrop-announces-series-50m-total-190300152.html

Unrelated "firedrill" projects (to avoid confusion):
- https://github.com/FourCoreLabs/firedrill
- https://github.com/zhiqiao/firedrill
- https://github.com/SarthakRoy-1/FireDrill
- https://github.com/firedrill-tools/firedrill (returned 404 when fetched, although search engines still index it)
