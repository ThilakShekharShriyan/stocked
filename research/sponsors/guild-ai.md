# Guild AI (guild.ai) — sponsor research

Researched 2026-11-19 for the Personal Agents Hackathon (2026-11-20, San Francisco). Challenge: "build a shopping agent with an audit trail". Guild prize: "Most Interesting Use of Agents" (100M / 50M / 25M tokens).

Confidence labels used below: **verified** = read on a primary page; **unverified** = only from aggregators or inferred; **not found** = searched and found nothing.

---

## 1. TL;DR

- **What it is:** Guild is a hosted "control plane for AI agents": you define agents (prompt + tool list, or TypeScript, or bring LangGraph/Goose/OpenClaw), Guild runs them in network-isolated runtimes, brokers every credential and LLM call server-side, enforces ALLOW/DENY policies and budgets, and records every LLM call and tool call in a per-session event log ([docs home](https://docs.guild.ai/), [security architecture](https://docs.guild.ai/platform/security-architecture.md)).
- **It is a new company, not the old ML tool:** this is Guild.ai Inc. (CEO James Everingham, ex-Meta; $44M from GV and others, March 2026). It is *not* Guild Education, and it is a different, newer company from Garrett Smith's 2016-2018 Chicago open-source experiment tracker "Guild AI" (`pip install guildai`), although it now uses the same guild.ai domain. Details in section 3.
- **What the prize wants:** the wording is only "Most Interesting Use of Agents". No criteria are published. Inferred from their product and past events: agents that actually run *on Guild*, more than one agent, doing a real action through governed integrations, with the trace visible.
- **Fastest qualifying path (about 30-45 min):** sign in at app.guild.ai, create a workspace, `npm install -g @guildai/cli`, `guild agent init --agent-type guild_native`, write `PROMPT.md` + `guild.yaml`, `guild agent save --wait --publish`, then call it from your app through an API trigger (`POST https://api.guild.ai/v1/workspaces/<owner>/<workspace>/sessions`). The response's `session_url` is a ready-made audit trail page.
- **The natural fit:** the hackathon asks for an audit trail; Guild's core feature is one. Use Guild's session event log (`llm_call`, `tool_call` events) plus a credential policy that blocks the purchase tool unless conditions are met, and a `ui_prompt` human approval before checkout.

---

## 2. What they do

### Product

Guild describes itself as "a control plane for AI agents. Teams use it to build, run, and govern agents across shared workspaces, including which services, models, and spend each agent is allowed" ([docs home](https://docs.guild.ai/)). The marketing site calls it a model-neutral control plane for engineering teams, with cost control, agent inventory, scoped credentials, policies, human approval gates, an MCP Gateway, observability and a "Software Factory" ([guild.ai](https://guild.ai/)).

Request flow, from the docs home page:

```text
Browser/CLI → Guild API → runtime (agent executor) → Agent
                    ↓
               Database
```

Main surfaces (all verified in docs):

| Surface | What it is | Source |
|---|---|---|
| Web app | app.guild.ai — workspaces, chat, agents, triggers, credentials, event log | [quickstart](https://docs.guild.ai/quickstart.md) |
| The Smith | Built-in agent (`guildai~the-smith`), default chat at `/chat`; answers questions, builds and debugs agents, sets up integrations and triggers | [What is The Smith?](https://docs.guild.ai/platform/smith.md) |
| CLI | `@guildai/cli` on npm (latest 0.32.0 on 2026-11-19, first published 2026-03-06, per the npm registry) | [CLI getting started](https://docs.guild.ai/cli/getting-started.md) |
| Agent SDK | `@guildai/agents-sdk` (TypeScript): `llmAgent`, `agent`, `guildTools`, `userInterfaceTools`, `pick` | [SDK tools](https://docs.guild.ai/sdk/tools.md) |
| Public REST API | `https://api.guild.ai/v1`, OpenAPI at docs.guild.ai/openapi.json, WebSocket for session events | [Agent sessions API](https://docs.guild.ai/api-reference/conversations.md) |
| MCP | Hosted MCP server `https://api.guild.ai/mcp` (OAuth), local `guild mcp`, docs MCP `https://docs.guild.ai/mcp` | [llms.txt](https://docs.guild.ai/llms.txt) |
| MCP gateway | One endpoint per "profile" exposing a workspace's integrations to any MCP client, with credentials, policies and audit trail behind it | [MCP gateway](https://docs.guild.ai/platform/mcp-gateway.md) |
| Agent Hub | Public catalog of agents and integrations, app.guild.ai/hub | [Explore the Agent Hub](https://docs.guild.ai/platform/agent-hub/explore.md) |
| Integrations | About 90 connectors, plus custom integrations for any HTTP API | [llms.txt](https://docs.guild.ai/llms.txt), [Create an integration](https://docs.guild.ai/services/create-an-integration.md) |

### "Smith bot" (the resource on the event page)

The event page lists "Smith bot" with the link https://app.guild.ai/chat and no description ([event page](https://tokensand.com/personalagenthack)). This is **The Smith**, Guild's built-in support agent: "It answers questions, helps build and debug agents, sets up integrations, and commissions Software Factories" and is the default landing chat when you open a workspace ([smith](https://docs.guild.ai/platform/smith.md), [getting started with The Smith](https://docs.guild.ai/platform/smith-getting-started.md)). Practical meaning: you can ask Smith in chat to build your agent for you. Smith "asks before it writes" for policy changes and has read access to workspace config.

### Agent definition formats

Five agent types ([agent types](https://docs.guild.ai/guide/agent-types.md), [guild.yaml reference](https://docs.guild.ai/guide/guild-yaml.md)):

1. **Native** — `PROMPT.md` (system prompt) + optional `guild.yaml` (tools). No code, no container, no build; Guild runs the loop. Text in, text out. Docs: "the type to reach for first" ([native agents](https://docs.guild.ai/guide/native-agents.md)).
2. **TypeScript** — `agent.ts` using `llmAgent({...})` (prompt-driven), or `agent({...})` with a `"use agent"` directive (deterministic code, auto-managed state), or self-managed state. Needed for structured input, attachments, per-call model preferences ([LLM agents](https://docs.guild.ai/guide/llm-agents.md)).
3. **Goose** — `recipe.yaml` + `guild.yaml`, runs in a container, supports a response JSON schema.
4. **OpenClaw** — `AGENTS.md` + `guild.yaml`, coding toolchain built in.
5. **LangGraph** — bring a Python LangGraph/LangChain agent.

`guild.yaml` example from the docs:

```yaml
integrations:
  - name: acme~github
    version: ^1.4.0
    tools: [github_repos_get, github_issues_list]

sub_agents:
  - name: acme~research
    version: ^1.0.0

builtins:
  - name: ui
```

Sub-agents are first-class: each declared sub-agent becomes a tool the parent can call, locked to a version at parent build time ([guild.yaml reference](https://docs.guild.ai/guide/guild-yaml.md)).

### Runtime and security model

- **Network isolation:** "Agents have no direct route to the internet." `fetch` cannot connect; `axios` and `node:http` cannot be imported. Every external call must go through a Guild integration (`task.tools.*`) ([SDK introduction](https://docs.guild.ai/guide/sdk-introduction.md)).
- **Credential proxy:** credentials are stored by Guild and attached server-side; agent code never sees them ([docs home](https://docs.guild.ai/)).
- **Credential policies:** ALLOW/DENY rules by operation (glob patterns), resource (repo, channel, domain, HTTP method), agent and workspace. DENY wins. New credentials start allow-all ([credential policies](https://docs.guild.ai/platform/credential-policies.md)).
- **Event log:** per session, "LLM calls, tool invocations, sub-task spawns, container log lines, errors, and lifecycle transitions ... primarily useful for debugging and auditing agent behavior". Event types include `llm_call`, `tool_call`, `agent_console`, `container_log`, `error` ([sessions](https://docs.guild.ai/platform/sessions.md)).
- **Org audit log:** a separate, immutable record of *administrative* actions, exportable to CSV ([audit logs](https://docs.guild.ai/insights/audit-logs.md)). Do not confuse the two in a demo: the per-session event log is the agent's decision trail.
- **Triggers:** event (Slack, GitHub, Jira ...), schedule (hourly to monthly), and API triggers ([triggers](https://docs.guild.ai/platform/triggers.md)).
- **Human in the loop:** `ui_prompt` "asks the user a question. Suspends the task until they answer"; Native agents also get `ui_progress` ([native agents](https://docs.guild.ai/guide/native-agents.md)).
- **Evaluations:** spec of samples, checks (`hasToolCall`, `toolCallContains`, `outputMatches`, `llmJudge`) and epochs. "Evaluations are gated per account" ([evaluations](https://docs.guild.ai/platform/evaluations.md)).
- **Observability export:** traces and metrics to Datadog or Splunk ([observability](https://docs.guild.ai/platform/observability.md)).

### Model access and what "tokens" means as a prize

- Two modes: **Managed** ("LLM usage draws from your **Guild tokens balance**") and **BYOK** (add any provider key and "your Guild tokens balance is not consumed"). Adding your first provider credential switches the *whole account* to BYOK automatically ([Models & providers](https://docs.guild.ai/platform/llm-settings.md)).
- Managed publishers and default models listed in the docs: Anthropic `claude-sonnet-4-6`, OpenAI `gpt-4o`, Gemini `gemini-3.5-flash`, Meta `muse-spark-1.1`, plus DeepSeek, Qwen, Moonshot, Z.ai via Bedrock / Fireworks / OpenRouter ([Models & providers](https://docs.guild.ai/platform/llm-settings.md)).
- So the prize ("100M tokens") is almost certainly **Guild tokens balance, i.e. managed LLM usage credit on the Guild platform**, not cash and not provider API credits. This reading is inferred from the docs; the exact exchange rate between a "Guild token" and model tokens is **unverified**. The same "100M tokens" prize appeared at Guild's own Hack of Horrors event ([Luma](https://luma.com/guild-xk38)). For scale, the default per-execution runaway limit is 50,000,000 LLM tokens ([execution limits](https://docs.guild.ai/reference/limits.md)).

### Positioning and competitors

- Positioning: neutral (model-, vendor-, framework-agnostic) governance and runtime layer; "GitHub for agents" via the Agent Hub ([Series A post](https://www.guild.ai/blog/news/guild.ai-raises-a-series-a), [funding post](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane)).
- Competitors Guild names itself: the marketing site has comparison pages against Claude Managed Agents and AWS agent services ([guild.ai](https://guild.ai/)); the docs ship migration guides from Arcade, Claude Managed Agents, CrewAI, Cursor Agent SDK, LangChain/LangGraph, OpenAI Agents API, Runlayer and Vercel eve ([llms.txt](https://docs.guild.ai/llms.txt)).
- Blog themes: "Why AI observability is not AI governance", "Your authorization platform doesn't govern your agents", "Least privilege for AI agents" ([blog](https://guild.ai/blog)). Their message is governance over raw capability.

---

## 3. Company facts

### Identity: which "Guild" is this?

| Entity | What | Relation |
|---|---|---|
| **Guild.ai Inc.** (this sponsor) | Agent control plane, San Francisco, founded 2025 (unverified year; GitHub org `guildaidev` was created 2025-06-02, [GitHub API](https://api.github.com/orgs/guildaidev)) | The sponsor |
| **Guild AI (Garrett Smith)** | Open-source ML experiment tracking toolkit. "Founded by Garrett Smith in 2018 ... headquartered in Chicago" ([old about page mirror](https://www-guildai.netlify.app/about/)). GitHub org `guildai` created 2016-06-16; repo `guildai/guildai` "Experiment tracking, ML developer tools", 909 stars, last push 2025-04-29 ([GitHub API](https://api.github.com/orgs/guildai)). PyPI package `guildai` ([PyPI](https://pypi.org/project/guildai/0.8.1)) | **Different, older project.** The new company now operates the guild.ai domain the old project used (the old GitHub org's website field still points to guild.ai) |
| **Guild Education** (guild.com) | Workforce education benefits company | Unrelated |
| GUILD (YC) | Defense manufacturing | Unrelated |

Conclusion: **a different, newer company that took over the name and domain.** How the domain/name changed hands (purchase, acqui-hire, other) is **not found**; no primary source describes it, and Guild's own about page does not mention the older project ([about](https://www.guild.ai/about)). A search snippet from Crunchbase lists Garrett Smith's founder role as ending May 2024 (unverified, page not readable directly).

Warning on bad data: several aggregators merge the two entities — Dealroom says the firm "started as an open-source toolkit for ... machine learning experiments" and is Chicago-based ([Dealroom](https://app.dealroom.co/companies/guild_ai_1)); Crunchbase and recentfunding.com list Garrett Smith as founder ([Crunchbase](https://www.crunchbase.com/organization/guild-ai), [recentfunding](https://recentfunding.com/companies/guild-ai/)). Even two Luma event blurbs for Guild-partnered hackathons describe Guild.ai as "experiment tracking and automation for machine learning" ([Agents you Love 2](https://luma.com/8lez4n99), [Memory Meets Motion](https://luma.com/5w09jxoy)). Treat all of that as stale boilerplate. **Do not `pip install guildai`** — that is the old tool. The sponsor's packages are npm `@guildai/cli` and `@guildai/agents-sdk`.

### People

- **James Everingham, CEO / co-founder.** Nearly a decade at Meta, led Instagram engineering, then ran Dev Infra (about 1,000 people serving about 40,000 engineers); led the work that became DevMate, Meta's internal agent platform; earlier worked on Mozilla at Netscape ([Dev Interrupted, 2026-03-10](https://linearb.io/dev-interrupted/podcast/james-everingham-guild-ai-control-plane-agents)).
- **Chris Waterson, CTO** ([about](https://www.guild.ai/about)). Described as co-founder by third-party directories ([agentcommunity.org](https://agentcommunity.org/m/guildai)) — co-founder status unverified on a primary page; background not found.
- Leadership page also lists: Val Bays (Chief Design Officer), Vincent Durmont (Principal Software Engineer), Killian Murphy (VP Engineering), Jen Rettig (Chief People Officer), Christina Smedley (CMO), Cristina Vasile (General Counsel), Andy Volkmann (CFO), Stephanie Harlow (Business Operations Manager) ([about](https://www.guild.ai/about)).
- Founding team "came from Meta's Dev Infra team" ([funding post](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane)).

### Funding

- **$44M total: $14M seed + $30M Series A, both led by GV, closed within four months**, announced 2026-03-03 ([Guild blog](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane), [SiliconANGLE](https://siliconangle.com/?p=733000)).
- Other investors: NFX, Khosla Ventures, Scribble Ventures, Acrew Capital, Webb Investment Network ([Guild blog](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane), [about](https://www.guild.ai/about)).
- **$300M valuation**: reported by Axios Pro citing Everingham ([Axios](https://www.axios.com/pro/enterprise-software-deals/2026/03/03/guildai-khosla-agentic-enterprise-tech-ai)) — seen only via search snippet, article itself paywalled (403), so treat as reported, not confirmed.
- GV's seed announcement post is dated 2025-11-19 ([GV](https://gv.com/news/guild-ai-sharing-intelligence)).

### HQ and size

- San Francisco (press release dateline, 2026-04-29, [Yahoo Finance / GlobeNewswire](https://finance.yahoo.com/sectors/technology/articles/guild-ai-introduces-first-control-171600318.html)). Headcount: not found on a primary source; aggregators say 11-50 (unverified).

### Timeline

| Date | Event | Source |
|---|---|---|
| 2025-11-19 | GV publishes seed announcement | [GV](https://gv.com/news/guild-ai-sharing-intelligence) |
| 2026-03-03 | $44M announced; waitlist open | [Guild blog](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane) |
| 2026-04-28/29 | Platform publicly available ("first control plane for AI agents") | [Series A post](https://www.guild.ai/blog/news/guild.ai-raises-a-series-a), [press release](https://finance.yahoo.com/sectors/technology/articles/guild-ai-introduces-first-control-171600318.html) |
| 2026-08-12 | Joins the Linux Foundation's Agentic AI Foundation | [blog index](https://guild.ai/blog) |
| 2026-08-18 | BasicOps customer story | [BasicOps case study](https://guild.ai/blog/customer-stories/basicops-case-study) |
| 2026-10-29 | SOC 2 Type II completed | [blog index](https://guild.ai/blog) |
| 2026-11-13 | "Your agents, your tools: Guild's growing integration catalog" | [blog index](https://guild.ai/blog) |
| Undated (2026) | Public API launch, Software Factory, Optimizer, OpenClaw + LangChain support, Datadog export, GitHub publish action | [blog index](https://guild.ai/blog) |

---

## 4. Customers and use cases

- **Named in homepage testimonials:** Turo (SVP Engineering), Sovrn (CTO), WorkWhile (Head of Engineering) ([guild.ai](https://guild.ai/)).
- **Case study:** BasicOps — a workplace product that treats "Agent Users" as a seat type and surfaces agents from partner platforms, with Guild as the highlighted partner; no quantitative metrics in the story ([BasicOps](https://guild.ai/blog/customer-stories/basicops-case-study)).
- **Demo repo:** `guildai-demo/sony-metagen-demo` exists on GitHub ([GitHub API](https://api.github.com/orgs/guildai-demo/repos)); whether Sony is a customer is **unverified** — do not claim it.
- The funding post cites anonymized results ("64% ticket reduction", "$18,000 in monthly savings") with no named customer ([Guild blog](https://guild.ai/blog/news/guild-raises-44m-agent-control-plane)).
- **Verticals / use cases they document:** engineering-team work — GitHub PR review, Slack mention answers, daily issue digests, Zendesk reply drafts, CI release notes ([recipes in llms.txt](https://docs.guild.ai/llms.txt)); the Software Factory (labeled issues to reviewed PRs, [factory](https://docs.guild.ai/platform/factory.md)); example agents "Maya: Calendar chief of staff" and "Biblos: Slack librarian" ([examples](https://docs.guild.ai/examples/overview.md)).
- **Consumer shopping is not a use case they show anywhere.** A personal shopping agent is off their beaten path, which helps for "most interesting".

---

## 5. Why they are at this hackathon

Inferred, with the evidence:

1. **Developer adoption after an enterprise-first launch.** Docs state "There is no self-serve sign-up or free trial" and route new users to sales ([pricing](https://docs.guild.ai/pricing.md)). Hackathons are how individual developers get onto the platform. They want accounts created, agents published, and sessions run.
2. **Heavy hackathon cadence run by DevRel.** Since July 2026 Guild has presented or partnered on at least six Bay Area / online hackathons (section 9), with their Head of DevRel hosting several. They built hackathon management into their own Discord bot ([little-guy-discord-bot](https://github.com/guildaidev/little-guy-discord-bot)).
3. **Token prizes drive retained usage.** Paying in Guild tokens means winners keep running agents on Guild in managed mode.
4. **They want agents built *on* Guild, and published.** Recent launches push distribution: Agent Hub, a public API, a Claude plugin that builds Guild agents from Claude Code ([guild-claude-plugin](https://github.com/guildaidev/guild-claude-plugin)), a GitHub publish action, "bring your own agents" for OpenClaw/LangChain ([blog](https://guild.ai/blog)).
5. **Their thesis is "multiplayer" governed agents, not single-player demos.** Everingham: agent workflows should be multiplayer and governed through a central control plane ([Dev Interrupted](https://linearb.io/dev-interrupted/podcast/james-everingham-guild-ai-control-plane-agents)). The judge's own bio says he speaks about "AI agents operating in real infrastructure rather than demos" ([Sessionize](https://sessionize.com/waddingham/)).
6. **The prize wording is deliberately open** ("Most Interesting Use of Agents", plural) — they are not asking for "best use of feature X"; they want something memorable they can show off, which still visibly runs on Guild.

---

## 6. Prize track

- **Exact wording:** "Most Interesting Use of Agents" ([event page](https://tokensand.com/personalagenthack)).
- **Prizes:** 1st 100M tokens, 2nd 50M tokens, 3rd 25M tokens ([event page](https://tokensand.com/personalagenthack)).
- **Sponsor description on the event page:** "Infrastructure for building, evaluating, and deploying AI agents." Resources listed: Docs (docs.guild.ai) and "Smith bot" (app.guild.ai/chat).
- **Published judging criteria:** none ([Luma](https://luma.com/personalagenthack)).
- **Whether using Guild is mandatory to win the Guild prize:** not stated. Assume yes.

### Person attending

**Corbett "Cory" Waddingham — Head of Developer Relations, Guild.ai. Judge and speaker** ([Luma](https://luma.com/personalagenthack)).

- The event lists "Corbett"; his Sessionize profile and Luma host listings say "Cory Waddingham" with the same title, so these are the same person (Cory as short form — inferred) ([Sessionize](https://sessionize.com/waddingham/), [Luma](https://luma.com/5w09jxoy)).
- Based in Sacramento, California. Bio: "previously led enterprise LangSmith enablement at LangChain and production ML enablement at Pinecone, including RAG pipelines and observability. He writes and speaks about AI agents operating in real infrastructure rather than demos." Topics: agentic AI, LLMOps, LLM observability, developer tools, governance ([Sessionize](https://sessionize.com/waddingham/)).
- Hosts Guild's hackathons: Memory Meets Motion, Hack of Horrors, Agents you Love 2, AI Security Engineering Hackathon (section 9).
- What this implies for judging: an observability and governance person. He will look at traces. Show him the session event log, the policy that blocked something, and the approval step. A chat-only demo with no visible trace is the weakest thing to show him.
- No talk title is published for his slot.

Other Guild staff seen judging earlier hackathons (may or may not attend): Killian Murphy (VP Engineering), Tamao Nakahara (Director, Growth and Community), Elijah Seabock (Software Engineer) ([Harness Engineering Hack](https://luma.com/harnesshack), [Self-Evolving Agents Hackathon](https://luma.com/swarmhack)).

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

Ranking is my inference from product emphasis; the features themselves are verified.

1. **Multiple agents running on Guild, calling each other as `sub_agents`**, each with a narrow tool list. The prize says "Agents"; sub-agents are a first-class `guild.yaml` section ([guild.yaml](https://docs.guild.ai/guide/guild-yaml.md)).
2. **Credential policies that visibly constrain an agent** — e.g. a researcher agent that is GET-only, and a DENY rule the buyer agent hits. This is their headline differentiator ([credential policies](https://docs.guild.ai/platform/credential-policies.md)).
3. **The session event log as the audit trail**, in the Guild UI and/or pulled into your own UI via `GET /v1/sessions/{id}/events` or the WebSocket ([sessions](https://docs.guild.ai/platform/sessions.md), [sessions API](https://docs.guild.ai/api-reference/conversations.md)).
4. **A custom integration for your own API** (import an OpenAPI spec), so the "purchase" goes through Guild's proxy ([create an integration](https://docs.guild.ai/services/create-an-integration.md)).
5. **Human approval via `ui_prompt`** before the consequential action ([native agents](https://docs.guild.ai/guide/native-agents.md)).
6. **Triggers** — API trigger from your app, or a schedule trigger (price watch) so the agent is not just a chat window ([API triggers](https://docs.guild.ai/platform/api-triggers.md), [schedule triggers](https://docs.guild.ai/platform/schedule-triggers.md)).
7. **Publishing the agent** (and ideally to the Agent Hub) — matches their distribution push ([publish to the Agent Hub](https://docs.guild.ai/platform/publish-to-agent-hub.md)).
8. **Spend budget / model policy** on the workspace — "this agent can spend at most $X of LLM per month" is a nice touch for a shopping agent ([limits](https://docs.guild.ai/reference/limits.md)).
9. **MCP gateway** — if your harness lives elsewhere (e.g. Claude Code, your own app), routing tool calls through `gateway.guild.ai` still gets policies and audit ([MCP gateway](https://docs.guild.ai/platform/mcp-gateway.md)).
10. **Evaluations** (`hasToolCall`, `toolCallContains`) — impressive, but gated per account; may not be enabled ([evaluations](https://docs.guild.ai/platform/evaluations.md)).
11. **Building with The Smith or the Claude plugin** — worth mentioning in the demo, not a differentiator on its own.

---

## 8. Quickstart

All commands and code below are copied or minimally adapted from the official docs.

### Signup and access

- Sign in at https://app.guild.ai with Google, GitHub, or an email magic link; you land on the chat page (The Smith) ([quickstart](https://docs.guild.ai/quickstart.md)).
- **Gotcha:** docs say "There is no self-serve sign-up or free trial" and "If you don't have one [an account], talk to sales" ([pricing](https://docs.guild.ai/pricing.md)). app.guild.ai/chat itself returned HTTP 200 unauthenticated when checked, but whether a brand-new account works without an invite is **unverified**. **Confirm access at the booth or in Guild's Discord (discord.gg/guild-ai, per [Hack of Horrors](https://luma.com/guild-xk38)) before kickoff.**
- Free tier / credits: nothing published. "Eligible accounts receive included Guild tokens" ([factory setup wizard](https://docs.guild.ai/platform/factory-setup-wizard.md)); whether hackathon accounts are "eligible" is unverified. Fallback: BYOK with your own Anthropic/OpenAI/OpenRouter key under Access & setup > Models & providers.

### In the web app (no CLI)

1. Workspaces > New Workspace.
2. Agents > Your agents > Create Agent > "Start with a prompt" — Guild scaffolds a Native agent and opens an editor with tabs for System Prompt (`PROMPT.md`), README and `guild.yaml` ([native agents](https://docs.guild.ai/guide/native-agents.md)).
3. Chat > type `@` and pick the agent.
4. Open the session's **Events** tab for the trail.

### CLI

Requires Node.js 22+ ([quickstart](https://docs.guild.ai/quickstart.md)).

```bash
npm install -g @guildai/cli
guild auth login            # opens browser; also configures npm registry for Guild packages
guild auth status           # ✓ Authenticated
guild setup                 # optional: installs Guild skills for Claude Code / Codex / Gemini
guild doctor                # if anything is off
```

Headless: `guild auth login --non-interactive` prints a URL; or set `GUILD_API_KEY`.

**Native agent (fastest):**

```bash
guild agent init --name my-native-agent --agent-type guild_native
```

Scaffold: `PROMPT.md`, `guild.yaml`, `README.md`, `.gitignore`. Then:

```bash
guild workspace select
guild agent test                                   # interactive chat
guild agent save --message "First version" --wait --publish
```

**TypeScript LLM agent:**

```bash
guild agent init --name hello-agent --template LLM
cd hello-agent
```

```typescript
import { llmAgent, guildTools } from "@guildai/agents-sdk"

export default llmAgent({
  description: "A friendly greeting agent",
  tools: { ...guildTools },
  systemPrompt: `You are a friendly assistant. Greet users warmly and answer their questions.`,
  mode: "multi-turn",
})
```

```bash
echo '{"prompt":"Hello!"}' | guild agent test --mode json
guild agent save --message "First version" --wait --publish
```

Notes from the docs: `--owner <name>` is required in non-interactive mode if your account belongs to an organization; every agent needs a category (`guild agent categories`); `description` is deprecated since SDK 0.4.0; Native/Goose agents skip build validation and go straight to `READY`.

**The official "paste into your coding agent" prompt** (from the quickstart) does the whole flow from Claude Code:

```text
Install the Guild CLI globally with npm: npm install -g @guildai/cli
Authenticate by running: guild auth login --non-interactive
This will print a URL. Tell me to open it in my browser to complete authentication, then wait for the command to finish.
After auth succeeds, run: guild setup --provider <claude, codex, or gemini — whichever you are> --yes
This will install skills for guild agent development and Factory setup.
Then select the home workspace: guild workspace select home
Then list agent categories with: guild agent categories
Create a new agent, picking a category from that list: guild agent init --name hello-agent --template LLM --category <category>
If I belong to an organization, ask me which account should own the agent and add --owner <name>.
Then change into the hello-agent directory it creates.
Edit agent.ts to be a friendly greeting agent using llmAgent with multi-turn mode and guildTools.
Test the agent with: echo '{"prompt":"Hello!"}' | guild agent test --mode json
Ask me if the response looks good. Once I confirm, save and publish with: guild agent save --message "First version" --wait --publish
```

**Claude Code plugin** ([guild-claude-plugin](https://github.com/guildaidev/guild-claude-plugin)):

```
/plugin marketplace add guildaidev/guild-claude-plugin
/plugin install guild@guild
```

Then `/guild:agent-development` with your agent idea. It connects Claude to `https://api.guild.ai/mcp`.

### Reaching the outside world from an agent

Agents cannot make direct HTTP calls. Options:

- **Public URL fetch** — integration `guildai~experimental-fetch` ([SDK introduction](https://docs.guild.ai/guide/sdk-introduction.md)):

```typescript
import { llmAgent, pick } from "@guildai/agents-sdk"
import { ExperimentalFetchTools } from "@guildai-services/guildai~experimental-fetch"

export default llmAgent({
  tools: {
    ...pick(ExperimentalFetchTools, ["experimental_fetch_fetch"]),
  },
  systemPrompt: "You fetch web pages and summarize them.",
})
```

It sends no credentials, default timeout 5 s, takes `url`, `method`, optional `data`, `headers`, `max_bytes`.

- **Catalog integrations relevant to shopping:** `guildai~stripe-openapi` (incl. Checkout Sessions, payment intents, refunds; `stripe_openapi_` tool prefix), `guildai~browserbase` (`browserbase_fetch_create`, `browserbase_search_web`), `guildai~firecrawl`, Tavily (`tavily-openapi`), Skyvern, Steel, Kernel, Pinecone, Supabase, Airtable, Gmail, Google Sheets, Twilio ([llms.txt](https://docs.guild.ai/llms.txt), [Stripe](https://docs.guild.ai/integrations/stripe-openapi.md), [Browserbase](https://docs.guild.ai/integrations/browserbase.md), [Firecrawl](https://docs.guild.ai/integrations/firecrawl.md)). Each needs your own API key connected under Access & setup > Credentials.

- **Custom integration for your own store API** ([create an integration](https://docs.guild.ai/services/create-an-integration.md)):

```bash
guild integration create my-service \
  --base-url https://api.example.com \
  --auth-scheme api-key \
  --description "Connect to the Acme API"

guild integration operation create myorg~my-service --openapi ./openapi.yaml
# or one at a time:
guild integration operation create myorg~my-service \
  --operation list_users --method GET --path /users --summary "List all users"
```

**Gotcha:** localhost and private ranges are blocked (SSRF protection). Expose a local mock store with ngrok/Localtunnel or deploy it. Publishing freezes the base URL — a changing ngrok URL means creating a new integration. Check the docs page for the publish step before relying on it.

### Policy (CLI form from the docs)

```bash
guild credentials policy create <credential-id> \
  --decision ALLOW \
  --operations "issues_*,pulls_*" \
  --resources '{"repos": ["acme/api"], "methods": ["GET"]}' \
  --agents <agent-id> \
  --workspaces <workspace-id>
```

Resource keys documented: GitHub `repos`, Slack `channels`, HTTP `domains`, any `methods`. There is no documented "amount <= $X" condition — a price cap must be enforced by operation/method/domain rules or in your own API ([credential policies](https://docs.guild.ai/platform/credential-policies.md)).

### Calling the agent from your app

**API trigger** (create in the web UI only: workspace > Triggers > Add Trigger > API; copy `<api_key_id>:<api_key_secret>`, shown once) ([API triggers](https://docs.guild.ai/platform/api-triggers.md)):

```bash
curl -X POST https://api.guild.ai/v1/workspaces/<owner_name>/<workspace_name>/sessions \
  -u "<api_key_id>:<api_key_secret>" \
  -H "Content-Type: application/json" \
  -d '{
    "session_type": "api_trigger",
    "agent_input": {
      "text": "Hello, agent!"
    }
  }'
```

Returns `201` with `id`, `session_url` (link to the session in the Guild UI), `root_task.status`, `token_usage`. Follow-up and status:

```bash
curl -X POST https://api.guild.ai/v1/sessions/<session_id>/events \
  -u "<api_key_id>:<api_key_secret>" \
  -H "Content-Type: application/json" \
  -d '{ "mode": "text", "content": "Follow-up question or answer goes here" }'

curl -X GET https://api.guild.ai/v1/sessions/<session_id> -u "<api_key_id>:<api_key_secret>"
```

**Account API key conversation** (Access & setup > API keys; scopes `sessions:write`, `workspaces:read`, `agents:read`) ([sessions API](https://docs.guild.ai/api-reference/conversations.md)):

```bash
curl -X POST https://api.guild.ai/v1/workspaces/acme~support/sessions \
  -u "<api_key_id>:<api_key_secret>" \
  -H "Content-Type: application/json" \
  -d '{
    "session_type": "chat",
    "agent_id": "<agent_id>",
    "initial_prompt": "Summarize open tickets tagged urgent."
  }'

curl -u "<api_key_id>:<api_key_secret>" \
  "https://api.guild.ai/v1/sessions/<session_id>/events?from_id=<last_seen_event_id>"

websocat -H "Authorization: Basic $(printf '%s' '<api_key_id>:<api_key_secret>' | base64)" \
  "wss://api.guild.ai/v1/sessions/<session_id>/events/ws?from_id=<last_seen_event_id>"
```

### Limits and gotchas (all from docs)

- No direct network from agent code; only `@guildai/agents-sdk` and `zod` resolve in the TS runtime ([SDK introduction](https://docs.guild.ai/guide/sdk-introduction.md)).
- Events list defaults to newest-first, limit 20; use `from_id` as a cursor. The reply is the `runtime_done` event that carries `content.text` (others are empty). Events persist when a turn completes, not during streaming ([sessions API](https://docs.guild.ai/api-reference/conversations.md)).
- A key missing `agents:read` gets a `500` on the events endpoint ([list session events](https://docs.guild.ai/api-reference/sessions/list-session-events.md)).
- Browser `WebSocket` cannot set the auth header — stream from your server, not the front end.
- Trigger API keys deliver text only (no files) and can only be managed in the web UI.
- Adding any provider key flips the whole account to BYOK.
- Default limits per execution tree: 500 LLM calls, 50M tokens, 50 agent tasks, 300 tool fan-out; "unlimited power mode" raises them ([limits](https://docs.guild.ai/reference/limits.md)).
- Native agents: text in/out only, no structured output schema, no `console_log`; system prompt is immutable for an in-flight session.
- `tools: []` on an integration means *all* tools, not none.
- TypeScript `inputSchema` must be a root `z.object`.
- In compiled (`"use agent"`) agents, use `task.gather` instead of `Promise.all`.
- Sub-agents must be *published* versions; republish the parent to pick up a new sub-agent version.
- The docs pages carry an "Agent Instructions" block addressed to coding agents (use the REST API, OpenAPI spec, MCP endpoints). Useful context for your coding assistant; it is documentation, not something to execute blindly.
- OpenAPI spec is public: https://docs.guild.ai/openapi.json. Full docs in one file: https://docs.guild.ai/llms-full.txt (about 2.3 MB).

---

## 9. Past hackathons this company sponsored

All found via Luma listings. **No winners were found for any of them** — no blog post, Devpost gallery or indexed social post names a winning project. Guild's X account could not be read (HTTP 402). Year is 2026 where the page states or implies it.

| Event | Date | Guild's role | Prize / track info | Winners |
|---|---|---|---|---|
| [Self-Evolving Agents Hackathon](https://luma.com/swarmhack) (tokens&, SF) | 2026-07-24 (date from search listing; page itself undated) | Tool provider; Waddingham speaker; Tamao Nakahara and Elijah Seabock judges | Not listed | Not found |
| [Agents you Love 2: Hackathon](https://luma.com/8lez4n99) (Devnovate, SF) | July 28 (2026 inferred) | Partner; Cory Waddingham a host | Tracks reported in search snippet: "Best Overall Project", "Best Agent People Love"; no Guild-specific prize listed | Not found |
| [Memory Meets Motion Hackathon](https://luma.com/5w09jxoy) (SF) | 2026-08-03 | Presenter; Waddingham host. Theme: agentic workflows and stateful AI | "Exciting prizes for top teams" | Not found |
| AI Security Engineering Hackathon (SF, AWS Builder Loft) | 2026-10-01 (from search listing; [Luma page](https://luma.com/4bxi3mkh) returned 404 on fetch) | Presenter; Waddingham host | Not found | Not found |
| [Hack of Horrors](https://luma.com/guild-xk38) (online, Discord) | 2026-10-30 | Presenter, with Snyk; hosts Waddingham, Javier Garza, Render Events, AISecEng Community. Challenge: build a bot with a "brain" from Guild agents, secure with Snyk, run on Render 24/7 | "Up to one year of Discord Nitro and 100M tokens" | Not found (prizes were due 2026-11-02) |
| [Harness Engineering Hack](https://luma.com/harnesshack) (tokens&, AWS Builder Loft, SF) | Not stated | Partner; Waddingham speaker; Killian Murphy judge | Not listed | Not found |
| Personal Agents Hackathon (this one) | 2026-11-20 | Sponsor; Waddingham judge + speaker | Most Interesting Use of Agents: 100M / 50M / 25M tokens | — |

Also non-hackathon: Guild presents at AI Infra Summit (2026-09-15 to 17, Santa Clara) and WeAreDevelopers World Congress ([AI Infra Summit sponsor page](https://www.ai-infra-summit.com/sponsor-directory/guildai)).

### Patterns in what wins

**Cannot be derived from evidence — no winner data exists publicly.** The honest substitutes:

- Guild repeatedly works with the same organizer (tokens&) and venue (AWS Builder Loft), and the same judge-speaker. Ask him directly what won last time; he hosted those events.
- The one Guild-authored challenge on record (Hack of Horrors) asked for an always-on bot whose "brain" is Guild agents, deployed for real — signalling they value agents that keep running after the demo, not one-shot scripts.
- The token prize is consistent (100M top prize at both events), so they are rewarding continued platform use.
- Their content is consistently about governance, least privilege and traceability, so a project that makes those visible aligns with what their team already believes.

---

## 10. Fit with "shopping agent with an audit trail"

The three challenge requirements map directly: "reliable context" = governed integrations and workspace context; "meaningful action" = a write call through the credential proxy; "show the information and steps" = the session event log.

Times assume one person on Guild and that account access works at kickoff. Add 20-30 min of first-time friction (auth, workspace, categories, owner flag).

### Idea A — Shallow: one Native agent behind an API trigger (30-45 min)

- `PROMPT.md` shopping agent + `guild.yaml` with `guildai~experimental-fetch` (or Tavily/Firecrawl) and `builtins: ui`.
- Your app calls the API trigger and embeds/link the returned `session_url` as "view audit trail".
- Qualifies; not memorable. It is one agent, and the prize says "agents".

### Idea B — Medium: governed purchase with human approval (1.5-2 h total)

- Deploy your mock store (catalog, cart, checkout) at a public URL; import its OpenAPI spec as a custom Guild integration.
- Agent uses `ui_prompt` to confirm the cart before calling checkout.
- Add a credential policy: DENY the checkout operation for every agent except the buyer; show a denied call in the event log during the demo.
- Demo beat: "the agent tried, Guild blocked it, here is the event".
- Risk: custom integration publishing and the frozen base URL; start this first.

### Idea C — Deep: a small "buying committee" of agents with separation of duties (3-3.5 h total)

- **Researcher** (read-only: search/fetch + your catalog GET operations), **Policy checker** (reads return/shipping policy context — e.g. from Senso via a custom integration — and vetoes), **Buyer** (the only agent whose policy allows the checkout POST), plus a parent orchestrator that declares the three as `sub_agents`.
- Per-agent credential policies enforce the separation, so the audit trail shows *who was allowed to do what*, not just what happened.
- Your UI streams `GET /v1/sessions/{id}/events` (server-side WebSocket or polling) and renders `tool_call` / `llm_call` / sub-task events as a receipt-style timeline next to the order.
- Add a workspace spend budget so the LLM cost of the shopping run is itself capped and reported (`token_usage` on the session).
- This is the strongest candidate for "Most Interesting Use of Agents" and stacks with the Senso track (log Senso's `content_id` / `version_id` inside tool results so they appear in Guild events).
- Risk: sub-agents must each be published; three publish cycles; more moving parts.

### Idea D — Beyond chat: standing order / price-watch via schedule trigger (+30-45 min on top of B or C)

- A schedule trigger runs the researcher hourly with fixed input; when the price condition is met it asks for approval (or notifies through Gmail/Slack/Twilio integration) and the buyer completes the purchase.
- Directly answers the organizer's "beyond a chat window" framing and Guild's "agents in real infrastructure rather than demos" preference.
- Risk: you cannot wait an hour in a demo — trigger it manually via the API trigger and show the schedule configured. `ui_prompt` behaviour in non-chat (trigger) sessions is an open question (section 11).

### Idea E — Alternative architecture: keep your own harness, route tools through the MCP gateway (45-60 min)

- If the team's agent loop lives elsewhere (own app, another sponsor's harness), create a gateway profile and call tools through `https://gateway.guild.ai/workspaces/{owner}~{workspace}/mcp/{profile}`; Guild applies credential policies and "records what ran".
- Lower Guild depth: agents do not run on Guild, so this is weaker for a prize named "use of agents". Use only as a fallback if agent publishing is blocked.

**Recommendation:** build B first (it is a complete, safe submission), then grow it into C if B works by about the 2.5-hour mark; add D's schedule trigger as the closing demo beat. Always label the purchase as simulated/test mode, as the organizers ask.

---

## 11. Risks and open questions for the booth

Risks:

1. **Access.** Docs say no self-serve sign-up or free trial. If hackathon accounts need an invite or org, losing 30 minutes here is the biggest threat. Resolve before 11:00.
2. **Managed LLM balance.** Unknown whether new accounts have any Guild tokens. Have a provider key ready for BYOK.
3. **Network isolation.** Every external call needs an integration; nothing on localhost is reachable. A mock store must be publicly hosted before the agent can touch it.
4. **Custom integration lifecycle.** Publishing freezes the endpoint URL; ngrok URL changes force a new integration.
5. **Latency / streaming.** Events persist only when a turn completes, so a live "thinking" UI built on the events API will update in steps, not token by token.
6. **Evaluations may not be enabled** on the account.
7. **Time split.** With 10 sponsors, going deep on Guild (idea C) consumes one person for most of the build.
8. **No public rubric and no winner precedent**, so "interesting" is the judge's taste.

Questions to ask Corbett/Cory Waddingham:

1. How do we get accounts today — is there a hackathon org, invite link or token grant? Do we get managed model access, and for which publishers?
2. Must the agents *run on Guild* to be eligible, or does routing tools through the MCP gateway count?
3. What does "interesting" mean to you — what won at Hack of Horrors / Memory Meets Motion, and why?
4. What exactly is a prize "token" — Guild tokens balance? Any expiry? Split across team members or one org?
5. Does `ui_prompt` work in API-trigger and schedule-trigger sessions (can we answer it with `POST /sessions/{id}/events`)? Where does the prompt surface?
6. Can credential policies express a condition on request body values (e.g. order total), or only operation / method / domain?
7. Is there a way to embed or publicly share a session's event log (like artifact magic links), so judges can open our audit trail without a Guild login?
8. Is `evals` enabled for hackathon accounts?
9. Fastest way to register a custom HTTP integration today — CLI, UI, or ask Smith? Any known sharp edges?
10. Does publishing to the Agent Hub (public) need admin approval, and would it help our submission?

---

## 12. Sources

Event
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack
- /Users/thilakshekharshriyan/Projects/PersonalHack/research/00-event-brief.md

Guild docs (primary)
- https://docs.guild.ai/
- https://docs.guild.ai/llms.txt
- https://docs.guild.ai/llms-full.txt
- https://docs.guild.ai/openapi.json
- https://docs.guild.ai/quickstart.md
- https://docs.guild.ai/pricing.md
- https://docs.guild.ai/platform/smith.md
- https://docs.guild.ai/platform/smith-getting-started.md
- https://docs.guild.ai/platform/artifacts.md
- https://docs.guild.ai/platform/sessions.md
- https://docs.guild.ai/platform/triggers.md
- https://docs.guild.ai/platform/api-triggers.md
- https://docs.guild.ai/platform/schedule-triggers.md
- https://docs.guild.ai/platform/llm-settings.md
- https://docs.guild.ai/platform/credential-policies.md
- https://docs.guild.ai/platform/mcp-gateway.md
- https://docs.guild.ai/platform/evaluations.md
- https://docs.guild.ai/platform/observability.md
- https://docs.guild.ai/platform/security-architecture.md
- https://docs.guild.ai/platform/factory.md
- https://docs.guild.ai/platform/factory-setup-wizard.md
- https://docs.guild.ai/platform/api-keys.md
- https://docs.guild.ai/platform/agent-hub/explore.md
- https://docs.guild.ai/platform/publish-to-agent-hub.md
- https://docs.guild.ai/insights/audit-logs.md
- https://docs.guild.ai/cli/getting-started.md
- https://docs.guild.ai/guide/sdk-introduction.md
- https://docs.guild.ai/guide/agent-types.md
- https://docs.guild.ai/guide/native-agents.md
- https://docs.guild.ai/guide/llm-agents.md
- https://docs.guild.ai/guide/guild-yaml.md
- https://docs.guild.ai/sdk/tools.md
- https://docs.guild.ai/reference/limits.md
- https://docs.guild.ai/api-reference/introduction.md
- https://docs.guild.ai/api-reference/conversations.md
- https://docs.guild.ai/api-reference/sessions/list-session-events.md
- https://docs.guild.ai/services/create-an-integration.md
- https://docs.guild.ai/integrations/stripe-openapi.md
- https://docs.guild.ai/integrations/tavily-openapi.md
- https://docs.guild.ai/integrations/browserbase.md
- https://docs.guild.ai/integrations/firecrawl.md
- https://docs.guild.ai/examples/overview.md

Guild site, blog, code
- https://guild.ai/
- https://www.guild.ai/about
- https://guild.ai/blog
- https://guild.ai/blog/news/guild-raises-44m-agent-control-plane
- https://www.guild.ai/blog/news/guild.ai-raises-a-series-a
- https://guild.ai/blog/customer-stories/basicops-case-study
- https://guild.ai/blog/community/why-i-build-platforms-and-why-agents-need-one-now
- https://github.com/guildaidev/guild-claude-plugin
- https://github.com/guildaidev/little-guy-discord-bot
- https://api.github.com/orgs/guildaidev and /repos
- https://api.github.com/orgs/guildai and /repos (old project)
- https://api.github.com/orgs/guildai-demo/repos
- https://registry.npmjs.org/@guildai/cli

Press and third party
- https://siliconangle.com/?p=733000
- https://www.axios.com/pro/enterprise-software-deals/2026/03/03/guildai-khosla-agentic-enterprise-tech-ai (paywalled; headline/snippet only)
- https://gv.com/news/guild-ai-sharing-intelligence
- https://finance.yahoo.com/sectors/technology/articles/guild-ai-introduces-first-control-171600318.html
- https://linearb.io/dev-interrupted/podcast/james-everingham-guild-ai-control-plane-agents
- https://www.tbpndigest.com/story/2026-03-03/guild-ai-raises-44m-to-build-an-enterprise-agent-control-plane-think-github-but-for-ai-agents/transcript
- https://www.ai-infra-summit.com/sponsor-directory/guildai
- https://agentcommunity.org/m/guildai (co-founder attribution; unverified)
- https://app.dealroom.co/companies/guild_ai_1 (conflates old and new entity)
- https://www.crunchbase.com/organization/guild-ai (snippet only; conflates entities)
- https://recentfunding.com/companies/guild-ai/ (snippet only; conflates entities)

Old "Guild AI" project
- https://www-guildai.netlify.app/about/
- https://pypi.org/project/guildai/0.8.1

People
- https://sessionize.com/waddingham/

Past hackathons
- https://luma.com/swarmhack
- https://luma.com/8lez4n99
- https://luma.com/5w09jxoy
- https://luma.com/4bxi3mkh (404 on fetch; details from search listing only)
- https://luma.com/guild-xk38
- https://luma.com/harnesshack

Could not be read: https://x.com/GuildAi (402), https://www.cbinsights.com/company/guild-ai (403), LinkedIn profiles.
