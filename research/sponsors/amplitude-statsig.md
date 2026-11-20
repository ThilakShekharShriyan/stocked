# Amplitude + Statsig — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (Nov 20, 2026, SF). Challenge: "Build a shopping agent with an audit trail."
> Code samples are copied from the official docs (fetched as markdown on 2026-11-19); anything labelled "adapted" is my combination of documented calls and has not been run. Items marked **unverified** / **not found** could not be confirmed.

---

## 1. TL;DR

- **What it is.** Amplitude (Nasdaq: AMPL) is a product-analytics platform that in 2026 added **Agent Analytics** — sessions/turns/tool-call tracing for AI agents with automatic quality scoring, joined to ordinary product events ([docs](https://amplitude.com/docs/amplitude-ai/agent-analytics/overview)). Statsig is the feature-flag / experimentation platform, now with **Agent Experimentation** (prompt and model configs, evaluators, traces) ([docs](https://docs.statsig.com/agent-experimentation/overview)).
- **Why they are listed together.** OpenAI bought Statsig in Sept 2025 for ~$1.1B in stock; on **May 5, 2026** Amplitude announced a "strategic partnership" under which **Amplitude takes on Statsig's brand, customers, platform maintenance and roadmap**, while the original Statsig team stays at OpenAI ([Amplitude blog](https://www.amplitude.com/blog/amplitude-and-statsig-partnership), [MarTech](https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/)). In practice Statsig is now an Amplitude-run product, and Amplitude's Q3 2026 results list Statsig features as its own product highlights ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)).
- **What the prize wants.** The wording is only "Best Use of Amplitude or Statsig" ($1,000 / $500), tagline "Agent analytics, feature flags, and experimentation" ([event page](https://tokensand.com/personalagenthack)). No rubric is published. Inference: they want agents instrumented with Agent Analytics and/or behaviour controlled by Statsig flags/experiments, with a visible dashboard in the demo.
- **Fastest qualifying integration (~30 min).** `pip install amplitude-ai` (or `npm install @amplitude/ai @amplitude/analytics-node`), wrap the LLM client, run each shopping task inside `agent.session(...)`, wrap tools with `tool()`. Every agent step becomes an `[Agent]` event — which doubles as the audit trail the challenge demands.
- **Best way to win.** Use **both** products in the loop Amplitude itself documents: Statsig assigns the agent variant (prompt / model / tool strategy), the Amplitude AI SDK records that variant on every event via `experiments: [{ key, variant, provider: 'statsig' }]`, and the demo shows an Amplitude chart splitting task success / cost / purchase conversion by arm ([Run an agent experiment](https://amplitude.com/docs/amplitude-ai/agent-analytics/run-an-agent-experiment)).

---

## 2. What they do

### Amplitude

**Core product.** Event-based product analytics: SDKs / HTTP API send events tied to a `user_id`; the UI builds funnels, retention, cohorts, dashboards. The platform also bundles Session Replay, Feature Experiment (flags + A/B), Web Experiment, Guides and Surveys, Heatmaps ([docs index](https://amplitude.com/docs)).

**AI / agent-facing products (what matters here):**

| Product | What it is | Source |
|---|---|---|
| **Agent Analytics** | Measures AI agents you build. Hierarchy: **Session** (one unit of work) → **Turn** (user message, tool calls, AI response) → **Span** (one LLM call / tool / retrieval), with **Agent** as a dimension. Events are prefixed `[Agent]` and carry your product `user_id`, so they sit in the same project as product events. Pitch: "Observability tools tell you what your agent did. Agent Analytics tells you whether it worked for the user and what it did to your business." | [Overview](https://amplitude.com/docs/amplitude-ai/agent-analytics/overview) |
| Quality scoring | **Signals** (automatic, per closed session: task completion, response quality, user intent, session safety, user friction, negative feedback, data quality, each with a written rationale); **Evaluators** (custom LLM/code judges you define and calibrate); **Scores** (thumbs/CSAT you send as `[Agent] Score`). | [Overview](https://amplitude.com/docs/amplitude-ai/agent-analytics/overview) |
| AI SDK | `@amplitude/ai` (Node) and `amplitude-ai` (Python). Wraps OpenAI, Anthropic, Azure OpenAI, Gemini, Bedrock, Mistral; integrations for LangChain, LlamaIndex, OpenAI Agents SDK, Claude Agent SDK, CrewAI (Python), LiveKit, Pipecat. Also accepts OpenTelemetry GenAI spans or raw HTTP events. | [SDK reference](https://amplitude.com/docs/sdks/agent-analytics/sdk), [Setup](https://amplitude.com/docs/amplitude-ai/agent-analytics/setup) |
| Agent experiments | The SDK stamps the flag variant on every `[Agent]` event (`[Agent] Experiments`); any chart splits by arm. Amplitude does not assign arms — it expects Amplitude Experiment, **Statsig**, or LaunchDarkly to do that. | [Track experiments](https://amplitude.com/docs/amplitude-ai/agent-analytics/track-experiments) |
| **Amplitude MCP Server** | Remote MCP (`https://mcp.amplitude.com/mcp`, OAuth) that lets Claude / Cursor / ChatGPT / Codex query and create charts, dashboards, cohorts, experiments, flags, taxonomy. Not an ingestion endpoint. | [MCP docs](https://amplitude.com/docs/amplitude-ai/amplitude-mcp) |
| Docs MCP | Public, unauthenticated docs search at `https://amplitude.com/docs/api/mcp`. | [MCP docs](https://amplitude.com/docs/amplitude-ai/amplitude-mcp) |
| **MCP Analytics** (Early Access) | SDK to instrument an MCP server *you* build; emits `[MCP]` events. Requires applying to the Early Access Program — not guaranteed available on a new account. | [MCP Analytics](https://amplitude.com/docs/amplitude-ai/mcp-analytics) |
| Amplitude's own agents | Global Agent + specialized agents (dashboard monitoring, session replay, experimentation, feedback), launched Feb 17, 2026; these analyse your Amplitude data, they are not a framework for building your agent. | [Press release](https://investors.amplitude.com/news-releases/news-release-details/amplitude-introduces-agentic-ai-analytics-next-era-product) |
| Wave (closed beta) | AI product agent that turns behavioural data into prioritized opportunities / PRDs; pairs with GitHub Copilot and Statsig in the "product learning loop". | [Blog, Oct 19 2026](https://amplitude.com/blog/amplitude-github-learning-loop) |
| AI Feedback, AI Visibility, AI Assistant, Session Replay, Guides and Surveys | Listed in the docs index; secondary for this build. Session Replay can be linked to agent sessions. | [docs index](https://amplitude.com/docs), [Setup](https://amplitude.com/docs/amplitude-ai/agent-analytics/setup) |

### Statsig

**Core product.** Server and client SDKs evaluate **feature gates** (boolean, off by default), **dynamic configs** (JSON values per user), **experiments / layers** (A/B/n with parameters) and **Autotune** (bandits); the SDK logs exposures and custom events, and the console computes metric lift. Evaluation is local after `initialize()` ([Python server SDK](https://docs.statsig.com/server-core/python-core), [Node server SDK](https://docs.statsig.com/server-core/node-core)). Also product analytics, session replay, and a warehouse-native deployment ([welcome](https://docs.statsig.com/welcome)).

**AI / agent-facing products:**

| Product | What it is | Source |
|---|---|---|
| **Agent Experimentation** | "Serves the prompts and models behind your AI app or agent, and grades their output before and after release." Parts: Configs (model + prompt + params, versioned; fetched at runtime), Playground, Datasets, Evaluators (LLM judge, Python, string compare, similarity; score 0–1), Offline evaluation, Runs, Online evaluation (grades sampled production spans ~1 minute after send), Traces. Console names differ from SDK names: configs = `getPrompt`, evaluators = graders. | [Overview](https://docs.statsig.com/agent-experimentation/overview) |
| AI SDKs | `statsig-ai` (Python, beta `0.0.1b10`) and `@statsig/statsig-ai` (Node, beta). Fetch configs, trace OpenAI / Anthropic / Gemini calls as OTel spans, log eval grades. | [Python AI SDK](https://docs.statsig.com/agent-experimentation/python), [Node AI SDK](https://docs.statsig.com/agent-experimentation/node) |
| **Statsig MCP** | `https://api.statsig.com/v3/mcp`; 18 default tools to read/create/update gates, experiments, dynamic configs, metrics, logs. | [MCP overview](https://docs.statsig.com/integrations/mcp/overview) |
| OpenAI integration | Log AI requests and experiment on prompts/models/params. | [docs](https://docs.statsig.com/integrations/openai) |

**Positioning of the pair.** Amplitude's stated thesis: AI made code cheap to write, but the lifecycle "remains bottlenecked in many other places" — evaluating before release, tracking after, rolling back, deciding what to build next ([MarTech quoting CEO Spenser Skates](https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/)). Statsig is the release/experiment control; Amplitude is the measurement and "what to build next" context.

**Competitors.** Named in sources: Optimizely (runs a "vs Statsig" campaign calling the deal "a race car without a driver" — [MarTech](https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/)), LaunchDarkly (named as an alternative flag tool in [Amplitude's own docs](https://amplitude.com/docs/amplitude-ai/agent-analytics/run-an-agent-experiment)), Convert and VWO (both published competitor commentary: [Convert](https://www.convert.com/blog/a-b-testing/statsig-moves-to-amplitude/), [VWO](https://vwo.com/blog/amplitude-statsig-partnership/)). From general industry knowledge (not sourced here): Mixpanel, PostHog, Heap, Pendo for analytics; Eppo/Datadog, GrowthBook for experimentation; LangSmith, Langfuse, Braintrust, Arize for LLM observability/evals.

---

## 3. Company facts

### Amplitude
- **Founders / HQ.** Spenser Skates (CEO) and Curtis Liu; Jeffrey Wang is also listed as co-founder (sources differ on whether he is a 2012 or 2014 co-founder, and on founding year 2012 vs a 2014 pivot from Sonalight). HQ San Francisco ([Wikipedia](https://en.wikipedia.org/wiki/Amplitude,_Inc.), [IVP](https://www.ivp.com/content/amplitude-making-an-increasingly-digital-first-world-a-better-place/)).
- **Public status.** Direct listing on Nasdaq as AMPL on Sept 28, 2021 ([Crunchbase News](https://news.crunchbase.com/news/amplitude-public-direct-listing-analytics-platform)).
- **Size (Q3 2026, reported Nov 4, 2026).** Revenue $110.6M (+25% YoY), ARR $431M (+24%), 855 customers with $100K+ ARR, "over 5,100 paying customers", FY2026 revenue guidance $415.8–418.2M ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)). Headcount: sources conflict (roughly 700–800) — unverified.
- **Notable 2026 news.**
  - Feb 17, 2026 — Global Agent + four specialized agents + MCP updates ([press release](https://investors.amplitude.com/news-releases/news-release-details/amplitude-introduces-agentic-ai-analytics-next-era-product)).
  - May 5, 2026 — Statsig partnership announced ([blog](https://www.amplitude.com/blog/amplitude-and-statsig-partnership)). A search summary reports Amplitude's CFO said it is accounted for as an asset acquisition, not a business combination ([search result citing longbridge/earnings coverage](https://longbridge.com/news/286149957.md) — I did not open the primary source; treat as secondary).
  - Aug 2026 — Agent Analytics generally available to all customers ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)).
  - Sept 2026 — "Headless Amplitude" on every plan: improved MCP server, developer CLI, service accounts ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)).
  - Oct 2026 — Wave + GitHub Copilot + Statsig "product learning loop"; Statsig Agent Experimentation expanded ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results), [blog](https://amplitude.com/blog/amplitude-github-learning-loop)).

### Statsig
- **Founder / HQ.** Founded 2021 by Vijaye Raji (ex-Facebook engineering leader); Bellevue, WA ([VentureBeat/Business Wire, May 6 2025](https://venturebeat.com/business/statsig-secures-100m-series-c-funding-to-transform-product-development-valuing-startup-at-1-1-billion/)).
- **Funding.** $100M Series C at $1.1B valuation, May 2025, led by ICONIQ Growth with Sequoia and Madrona; ~$40M annualized revenue at the time per Fortune ([Fortune](https://dc.fortune.com/2025/05/06/statsig-series-c-100-million-1-1-billion-eppo-datadog)).
- **Ownership timeline.**
  - Sept 2, 2025 — OpenAI announces acquisition, all-stock, ~$1.1B; Raji becomes OpenAI CTO of Applications; OpenAI says Statsig will "continue operating independently and serving its customer base out of its Seattle office" ([Convert summary citing CNBC](https://www.convert.com/blog/a-b-testing/statsig-moves-to-amplitude/), [SiliconANGLE](https://siliconangle.com/?p=709797)).
  - May 5, 2026 — "Amplitude will take on Statsig's brand and customers", maintain and develop the cloud and warehouse-native platform, and "begin building a more integrated roadmap"; it will "work closely with the Statsig team at OpenAI during the transition" ([Amplitude blog](https://www.amplitude.com/blog/amplitude-and-statsig-partnership)). OpenAI keeps the engineers, product leaders and statisticians ([MarTech](https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/)).
  - **Current state (Nov 2026):** Statsig is a product line operated by Amplitude; docs still live at docs.statsig.com and signup at console.statsig.com; statsig.com pages do not carry an "Amplitude company" tagline that I could find. Financial terms of the Amplitude deal: **not found**. Whether Statsig Inc. as a legal entity sits under OpenAI or Amplitude: **unverified**.

---

## 4. Customers & use cases

### Amplitude
- Named in the Q3 2026 boilerplate: Atlassian, Replit, The Economist, Square, Burger King, NBCUniversal ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)).
- Case studies on the customers page ([amplitude.com/customers](https://amplitude.com/customers)), relevant to commerce: **Jersey Mike's** (doubled mobile app orders with personalized messages), **Coca-Cola FEMSA** (retail growth through AI personalization), **DeFacto** (4x experimentation), **Tira**, **Ford**. AI-related: **ACKO** (+13% conversions with GenAI), **NTT DOCOMO**, **The Economist Group** ("quantified AI quality"). Others: Cisco, Hostinger (+20% conversion lift), NS, Temporal.
- Verticals: e-commerce/retail, media, fintech, B2B SaaS, consumer tech.

### Statsig
- Customers page ([statsig.com/customers](https://www.statsig.com/customers)): **Notion** (single-digit to hundreds of experiments per quarter), **Whatnot** (0 → 400 annual experiments), **OfferUp** (+11% conversions), **Bed Bath & Beyond**, **HelloFresh** (~1,000 experiments/yr, warehouse native), **Brex**, **Bluesky**, **Webflow**, **Lime**, **Captions**, **Cider**, **LAAM** (+75% conversions), **Secret Sales**, SoundCloud, Ancestry, Plex, Rec Room.
- **OpenAI** appears as a customer logo and in testimonials on the pricing page ([statsig.com/pricing](https://www.statsig.com/pricing)).
- Commerce-relevant pattern: marketplaces and retailers (Whatnot, OfferUp, Bed Bath & Beyond, Cider, LAAM, Secret Sales) use it to A/B test conversion.

---

## 5. Why they are at this hackathon (inference)

Evidence, then the inference:

- Agent Analytics went GA in **August 2026**; Headless Amplitude (MCP/CLI/service accounts on every plan) in **September 2026**; Statsig Agent Experimentation expanded in **October 2026** ([Q3 release](https://amplitude.com/press/third-quarter-2026-financial-results)). All three are developer-facing and new.
- The event tagline leads with "**Agent analytics**, feature flags, and experimentation" ([event page](https://tokensand.com/personalagenthack)) — the Luma listing says "Product analytics, feature flags, and experimentation" ([Luma](https://luma.com/personalagenthack)), so someone deliberately swapped in "Agent analytics" for the hackathon page.
- Free tiers are generous and credit-card-free on both sides (see section 8), and both products ship "paste this prompt into your coding agent" setup flows ([quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart)).
- Amplitude and Statsig are co-marketing in SF right now: "SHIP IT" party with an "AI product challenge" on Oct 28, 2026 and a Wave + Statsig demo at GitHub Universe on Oct 29, 2026 ([blog](https://amplitude.com/blog/amplitude-github-learning-loop)).
- The people sent are **Solutions Engineers** (pre-sales technical staff), not DevRel — one judge, two speakers.

**Inference:** They want AI-agent builders to (a) adopt Agent Analytics as the default way to measure an agent in production, positioned against pure LLM-observability tools, and (b) see Amplitude + Statsig as one stack — Statsig ships and randomizes agent variants, Amplitude measures the outcome. They also need proof points that the Statsig product is alive and integrated post-transition, given public scepticism ([MarTech](https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/)). A project that demonstrates the combined loop on a business outcome (a purchase) is the story their SEs tell customers.

---

## 6. Prize track

- **Exact wording:** "Best Use of Amplitude or Statsig" ([event page](https://tokensand.com/personalagenthack)).
- **Prizes:** 1st $1,000 · 2nd $500 ([event page](https://tokensand.com/personalagenthack)). Cash vs gift card: not stated.
- **Judging criteria:** none published. "or" means either product qualifies.
- **Links given by the organizers:** [amplitude.com/get-started](https://amplitude.com/get-started), [console.statsig.com/sign_up](https://console.statsig.com/sign_up), [Amplitude docs](https://amplitude.com/docs), [Statsig docs](https://docs.statsig.com/welcome).

**People** (all Amplitude; no Statsig-branded staff listed — [Luma](https://luma.com/personalagenthack)):

| Person | Role at event | Background found |
|---|---|---|
| **Rebecca ("Becca") Abu Sharkh** | Judge — "Solutions Engineering @ Amplitude" | Listed as **Senior Manager, Solutions Engineering** and a 2025 Founders' Award winner: "a cornerstone of Amplitude's Sales organization in 2025", "the technical lead on Amplitude's largest revenue deal to date" ([Founders' Awards post, Mar 6 2026](https://amplitude.com/blog/2025-founders-awards)). A search summary says the post credits her with helping launch a Strategic Solutions Engineering team (secondary; not re-read verbatim). A colleague's profile says she inspired them into solutions engineering and praises her "spirit of ownership" ([Meet the Ampliteer](https://amplitude.com/blog/meet-the-ampliteer-bethany-mangan)). Implication: she evaluates like an enterprise pre-sales lead — business outcome and a clean demo narrative over clever plumbing. |
| **Sid Nayak** | Speaker — "Principal Solutions Engineer @ Amplitude" | Aggregator profiles show a path at Amplitude from Account Executive (2021) → Solutions Engineer (2022) → Senior SE (2024); "Principal" title appears on Luma and one aggregator ([village.ai](https://village.ai/person/sid-nayak-NTExODY0ODEw), [theorg](https://theorg.com/org/amplitude/org-chart/sid-nayak)). Aggregator data — **unverified** in detail. No talks or posts found. |
| **Jackson Yang** | Speaker — "Solutions Engineering @ Amplitude" | **Not found** — no public background located beyond the event listing. |

Talk titles for the Amplitude speakers: not listed on the agenda ([Luma](https://luma.com/personalagenthack)).

---

## 7. What they expect to see used — ranked by likely impact on this sponsor

Ranking is my inference from section 5; the sponsor has published no rubric.

1. **Amplitude Agent Analytics via the AI SDK, fully instrumented** — sessions with real user/session IDs, wrapped LLM client, `tool()`-wrapped tools with input/output, `[Agent] Score` for user feedback. Newest flagship, and the tagline's first words. ([SDK](https://amplitude.com/docs/sdks/agent-analytics/sdk))
2. **Statsig-assigned variant recorded in Agent Analytics** (`experiments: [{ key, variant, provider: 'statsig' }]`) with a chart split by arm. This is literally the cross-product example in Amplitude's docs and the joint story. ([Track experiments](https://amplitude.com/docs/amplitude-ai/agent-analytics/track-experiments))
3. **Agent events joined to business events** — emit ordinary product events (`Product Viewed`, `Checkout Started`, `Order Placed`) with the same `user_id` and build a funnel from first agent session to purchase. The docs explicitly tell you to do this: "For business actions the agent performs, such as a purchase... emit your standard product events" ([HTTP path doc](https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-http-api)).
4. **Statsig feature gates as agent guardrails / kill switch** (e.g. gate real checkout vs simulated; dynamic config for spend limit) — cheap and maps to "audit trail" and safe actions. ([Python SDK](https://docs.statsig.com/server-core/python-core))
5. **Statsig Agent Experimentation** — prompt in a config fetched with `get_prompt()`, traces, `log_eval_grade()`, an evaluator. Highlighted in Amplitude's Q3 release, but the SDK is beta. ([Overview](https://docs.statsig.com/agent-experimentation/overview))
6. **Custom Evaluators in Amplitude** (e.g. "did the agent cite a source for every product claim?") — included on every plan per the plan table; requires adding your own model-provider key. ([Overview](https://amplitude.com/docs/amplitude-ai/agent-analytics/overview))
7. **Amplitude MCP / Statsig MCP** — used at build time from Claude Code/Cursor to create the dashboard, flags and experiment; or, more ambitious, the shopping agent queries Amplitude MCP for its own history. ([Amplitude MCP](https://amplitude.com/docs/amplitude-ai/amplitude-mcp), [Statsig MCP](https://docs.statsig.com/integrations/mcp/overview))
8. **Session Replay linked to agent sessions**, Guides and Surveys for post-purchase feedback — nice extras if there is a web UI. ([Setup](https://amplitude.com/docs/amplitude-ai/agent-analytics/setup))
9. Plain `track()` events only — qualifies, unlikely to place.

---

## 8. Quickstart

### 8.1 Signup, keys, free tiers

| | Amplitude | Statsig |
|---|---|---|
| Signup | [amplitude.com/get-started](https://amplitude.com/get-started) or [app.amplitude.com/signup](https://app.amplitude.com/signup) | [console.statsig.com/sign_up](https://console.statsig.com/sign_up) |
| Key | Project **API key** → `AMPLITUDE_AI_API_KEY`. For Amplitude Experiment, a **deployment key** (server deployment). | **Server Secret Key** from [console.statsig.com/api_keys](https://console.statsig.com/api_keys) → `STATSIG_SDK_KEY`. Console API key for MCP. |
| Free tier | 2M events/month, no card; **5,000 agent sessions/month**, chats retained 14 days; unlimited feature flags, 1 active Feature Experiment; 10,000 Session Replay sessions/month; 1 guide or survey; AI agents and MCP included ([pricing](https://amplitude.com/pricing), [plan table](https://amplitude.com/docs/amplitude-ai/agent-analytics/overview)) | Developer tier free, no card: 2M metered events/month, unlimited flag/config checks, experiments, **agent experimentation**, 50,000 session replays/month, unlimited seats ([pricing](https://www.statsig.com/pricing)) |

### 8.2 Amplitude Agent Analytics — AI SDK

Install ([quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart)):

```bash
# Node
npm install @amplitude/ai @amplitude/analytics-node
# Python  (provider extras exist, e.g. "amplitude-ai[openai,fastapi]"; `anthropic` is also an extra)
python3 -m pip install amplitude-ai
export AMPLITUDE_AI_API_KEY="YOUR_API_KEY"
```

Fastest route — paste into your coding agent (verbatim from the docs):

```text
Instrument this app with Amplitude Agent Analytics using the Python SDK.

Install the SDK:

python3 -m pip install amplitude-ai

Then run `amplitude-ai --print-guide` and follow the printed instrumentation guide.
```

```text
Instrument this app with Amplitude Agent Analytics using the Node SDK.

Fetch and follow https://raw.githubusercontent.com/amplitude/Amplitude-AI-Node/main/amplitude-ai.md
Then install the SDK as that guide says:

npm install @amplitude/ai @amplitude/analytics-node
```

Minimal manual session, no LLM wrapper (verbatim, [quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart)):

```python
import os
from amplitude_ai import AmplitudeAI

ai = AmplitudeAI(api_key=os.environ["AMPLITUDE_AI_API_KEY"])
agent = ai.agent("my-agent")

with agent.session(user_id="user-1", session_id="session-1") as session:
    session.track_user_message("What is my balance?")
    session.track_ai_message(
        "Your balance is $1,234.", "gpt-4o", "openai", 120
    )

ai.flush()
```

```typescript
import { AmplitudeAI } from "@amplitude/ai";

const ai = new AmplitudeAI({
  apiKey: process.env.AMPLITUDE_AI_API_KEY!,
});

const agent = ai.agent("my-agent");
const session = agent.session({ userId: "user-1", sessionId: "session-1" });

await session.run(async (s) => {
  s.trackUserMessage("What is my balance?");
  s.trackAiMessage("Your balance is $1,234.", "gpt-4o", "openai", 120);
});

await ai.flush();
```

Wrapped LLM client (verbatim, [instrument-ai-sdk](https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-ai-sdk)):

```python
import os
from amplitude_ai import AmplitudeAI, OpenAI

ai = AmplitudeAI(api_key=os.environ['AMPLITUDE_AI_API_KEY'])
client = OpenAI(amplitude=ai, api_key=os.environ['OPENAI_API_KEY'])
agent = ai.agent('my-agent')

with agent.session(user_id=user_id, session_id=session_id) as s:
    s.track_user_message(message)
    client.chat.completions.create(model=model, messages=messages)

ai.flush()
```

```typescript
import { AmplitudeAI, OpenAI } from "@amplitude/ai";

const ai = new AmplitudeAI({ apiKey: process.env.AMPLITUDE_AI_API_KEY });
const openai = new OpenAI({
  amplitude: ai,
  apiKey: process.env.OPENAI_API_KEY,
});
const agent = ai.agent("my-agent");

const session = agent.session({ userId, sessionId });
await session.run(async (s) => {
  s.trackUserMessage(message);
  return openai.chat.completions.create({ model, messages });
});
await ai.flush();
```

Other provider wrappers per the [SDK reference](https://amplitude.com/docs/sdks/agent-analytics/sdk): Python `Anthropic(amplitude=ai, api_key=...)`, `Gemini(amplitude=ai, api_key=...)`, `Bedrock(...)`, `Mistral(...)`; Node `new Anthropic({ apiKey, amplitude: ai })`, etc. Existing client: `wrap(existing_client, amplitude=ai)` (Python) / `wrap(existingClient, ai)` (Node). Tool-call tracking is supported for OpenAI, Anthropic, Azure OpenAI, Bedrock — **not** Gemini or Mistral.

Tools — this is the audit trail (verbatim, [SDK reference](https://amplitude.com/docs/sdks/agent-analytics/sdk)):

```python
from amplitude_ai import tool

search_products = tool(search_db, name="search_products")

# Inside the session context, call as usual:
result = search_products(query)
# [Agent] Tool Call event emitted with duration, success, input/output
```

```typescript
import { tool } from "@amplitude/ai";

const searchProducts = tool(searchDB, { name: "search_products" });

// Inside session.run, call as usual:
const result = await searchProducts(query);
// [Agent] Tool Call event emitted with duration, success, input/output
```

Inline alternative: `s.trackToolCall(name, latencyMs, success, { input, output })`. The option names must be exactly `input` / `output`; `toolInput` / `toolOutput` are silently ignored.

User feedback score (verbatim, [setup](https://amplitude.com/docs/amplitude-ai/agent-analytics/setup)):

```python
ai.score(
    user_id=user_id,
    session_id=session_id,
    name="user-feedback",
    value=0,
    target_id=ai_message_id,
    target_type="message",
    source="user",
)
```

Claude Agent SDK integration (verbatim, [SDK reference](https://amplitude.com/docs/sdks/agent-analytics/sdk)):

```python
from amplitude_ai.integrations.claude_agent_sdk import ClaudeAgentSDKTracker
from claude_agent_sdk import ClaudeAgentOptions, query

agent = ai.agent("code-reviewer")
tracker = ClaudeAgentSDKTracker()

async with agent.session(user_id="u1", session_id="sess-abc") as s:
    options = ClaudeAgentOptions(hooks=tracker.hooks(s))
    async for message in query(prompt="Analyze this codebase", options=options):
        tracker.process(s, message)
```

Verify: `amplitude-ai doctor` (Python) or `npx -y -p @amplitude/ai amplitude-ai doctor` (Node); construct the client with `debug: true` to print every event to stderr ([verify](https://amplitude.com/docs/amplitude-ai/agent-analytics/verify), [quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart)).

### 8.3 Amplitude — plain product events (purchase funnel)

Verbatim from [Install a server SDK](https://amplitude.com/docs/get-started/install-server-sdk):

```python
# pip install amplitude-analytics
from amplitude import Amplitude, BaseEvent

amplitude = Amplitude("AMPLITUDE_API_KEY")

amplitude.track(BaseEvent(
    event_type="Button Clicked",
    user_id="user@example.com"
))
```

```javascript
// npm install @amplitude/analytics-node
import { init, track } from "@amplitude/analytics-node";

init("AMPLITUDE_API_KEY");
track("Button Clicked", undefined, { user_id: "user@example.com" });
```

Python client batches: default flush every 10 s or 200 events; call `client.flush()` / `shutdown()` before exit ([Python SDK](https://amplitude.com/docs/sdks/analytics/python/python-sdk)).

### 8.4 Amplitude — raw HTTP (any runtime)

Verbatim from [instrument-http-api](https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-http-api):

```bash
curl -X POST https://api2.amplitude.com/2/httpapi \
  -H "Content-Type: application/json" \
  -d '{
    "api_key": "AMPLITUDE_API_KEY",
    "events": [{
      "event_type": "[Agent] User Message",
      "user_id": "user-123",
      "event_properties": {
        "[Agent] Session ID": "ticket-42",
        "[Agent] Turn ID": 1,
        "[Agent] Trace ID": "trace-abc",
        "[Agent] Agent ID": "support-bot",
        "[Agent] Message ID": "msg-1",
        "$llm_message": { "text": "I need a refund" }
      }
    }]
  }'
```

Required properties per event type are in the [taxonomy event contract](https://amplitude.com/docs/amplitude-ai/agent-analytics/taxonomy): Tool Call needs `[Agent] Invocation ID`, `[Agent] Tool Name`, `[Agent] Tool Success`, `[Agent] Latency Ms`, `[Agent] Parent Message ID`; AI Response needs `[Agent] Message ID`, `[Agent] Model Name`, `[Agent] Provider`, `[Agent] Latency Ms`, token counts.

### 8.5 Statsig — gates, configs, experiments, events (server)

Python (verbatim fragments from [python-core](https://docs.statsig.com/server-core/python-core)):

```python
# pip install statsig-python-core
from statsig_python_core import Statsig, StatsigOptions, StatsigUser  # import uses underscores

statsig = Statsig("secret-key")
statsig.initialize().wait()

user = StatsigUser("a-user")

if statsig.check_gate(user, "a_gate"):
    ...  # Gate is on, enable new feature

config = statsig.get_dynamic_config(StatsigUser("my_user"), "a_config")
product_name = config.get_string("product_name", "Awesome Product v1")
price = config.get_float("price", 10.0)

title_exp = statsig.get_experiment(StatsigUser("my_user"), "new_user_promo_title")
title = title_exp.get_string("title", "Welcome to Statsig!")

statsig.log_event(
    user=StatsigUser("user_id"),
    event_name="add_to_cart",
    value="SKU_12345",
    metadata={
        "price": "9.99",
        "item_name": "diet_coke_48_pack"
    }
)

statsig.shutdown().wait()  # flushes event logs
```

Node (verbatim fragments from [node-core](https://docs.statsig.com/server-core/node-core)):

```typescript
// npm i @statsig/statsig-node-core
import { Statsig, StatsigUser } from '@statsig/statsig-node-core';

const statsig = new Statsig("secret-key");
await statsig.initialize();

const user = new StatsigUser({ userID: "a-user" });

if (statsig.checkGate(user, "a_gate")) {
    // Gate is on, enable new feature
}

const config = statsig.getDynamicConfig(user, "a_config");
const price = config.getValue("price", 10.0);

const titleExp = statsig.getExperiment(user, "new_user_promo_title");
const experimentTitle = titleExp.getValue("title", "Welcome to Statsig!");

statsig.logEvent(user, "add_to_cart", null, { price: "9.99", item_name: "diet_coke_48_pack" });

await statsig.shutdown();
```

### 8.6 The joint pattern — Statsig assigns, Amplitude records (adapted)

The Amplitude docs give this shape with a placeholder flag client and `provider: "statsig"` ([track-experiments](https://amplitude.com/docs/amplitude-ai/agent-analytics/track-experiments)):

```python
variant = flags.get_variant("planner")  # your existing flag client
agent = ai.agent("support-bot", user_id=user_id)

with agent.session(
    session_id=session_id,
    experiments=[{"key": "planner", "variant": variant, "provider": "statsig"}],
) as s:
    ...  # provider wrappers, tools, and tracking calls in the session all carry the experiment
```

**Adapted** (my combination of the two documented APIs; not run — requires `amplitude-ai` >= 1.22.0 or `@amplitude/ai` >= 0.25.0):

```python
exp = statsig.get_experiment(StatsigUser(user_id), "shopper_prompt")   # Statsig assigns + logs exposure
variant = exp.get_string("prompt_style", "detailed")                    # read a parameter, not the group name

with agent.session(
    user_id=user_id,
    session_id=session_id,
    experiments=[{"key": "shopper_prompt", "variant": variant, "provider": "statsig"}],
) as s:
    s.track_user_message(task)
    ...  # run the agent with the chosen prompt
```

Every event then carries `[Agent] Experiments` (e.g. `["shopper_prompt:concise"]`) and `[Agent] Experiment Variant`; group any Amplitude chart by it. The Amplitude SDK never sends an exposure event — Statsig keeps that job. Units: `agent_session` (default), `turn`, `span` ([run-an-agent-experiment](https://amplitude.com/docs/amplitude-ai/agent-analytics/run-an-agent-experiment)). Statsig's own guidance is to read parameters rather than group names ([docs](https://docs.statsig.com/experiments/implementation/getting-group)).

Alternative using Amplitude Experiment instead of Statsig (verbatim, same page):

```python
from amplitude_ai import AmplitudeAI, experiments_from_amplitude
from amplitude_experiment import Experiment, User

ai = AmplitudeAI(api_key=os.environ["AMPLITUDE_API_KEY"])
experiment = Experiment.initialize_remote(os.environ["AMPLITUDE_DEPLOYMENT_KEY"])
agent = ai.agent("support-bot")

def handle_conversation(user_id: str, session_id: str) -> None:
    variants = experiment.fetch_v2(User(user_id=user_id))
    prompt = CONCISE_PROMPT if variants["planner-prompt"].value == "concise" else DETAILED_PROMPT

    with agent.session(
        user_id=user_id,
        session_id=session_id,
        experiments=experiments_from_amplitude(variants, keys=["planner-prompt"]),
    ) as s:
        run_turns(s, user_id, prompt)
```

### 8.7 Statsig Agent Experimentation (beta SDK)

Verbatim from the [Python AI SDK page](https://docs.statsig.com/agent-experimentation/python):

```python
# pip install statsig-ai==0.0.1b10   (Python 3.10+)
import os
from statsig_ai import StatsigAI, StatsigCreateConfig, StatsigOptions, StatsigUser

options = StatsigOptions()
statsig_ai = StatsigAI.new_shared(
    StatsigCreateConfig(sdk_key=os.environ["STATSIG_SDK_KEY"], statsig_options=options)
)
statsig_ai.initialize()

user = StatsigUser(user_id="user-123")
prompt = statsig_ai.get_prompt(user, "support_bot")
live = prompt.get_live()
model = live.get_model(fallback="gpt-6.1-terra")
messages = live.get_prompt_messages({"question": "How do I reset my password?"})

statsig_ai.log_eval_grade(
    user,
    live,
    1,
    "user_feedback",
    {"session_id": conversation_id, "reasoning": "The user rated the reply as helpful."},
)
```

Tracing: call `initialize_tracing(InitializeTracingOptions(service_name=..., environment=...))` once, then `wrap_openai(OpenAI(), options)` / `wrap_anthropic(Anthropic(), options)` / `wrap_gemini(...)`. Node equivalents: `npm install @statsig/statsig-ai @statsig/statsig-node-core`, `StatsigAI.newShared({ sdkKey })`, `getPrompt`, `initializeTracing`, `wrapOpenAI`, `wrapAnthropic` ([Node AI SDK](https://docs.statsig.com/agent-experimentation/node)).

### 8.8 MCP setup (Claude Code)

```bash
# Amplitude (OAuth; then run /mcp in Claude Code and sign in)
claude mcp add --transport http --scope user Amplitude https://mcp.amplitude.com/mcp

# Statsig (OAuth)
claude mcp add --transport http statsig https://api.statsig.com/v3/mcp

# Statsig with a Console API key instead
claude mcp add --transport http statsig-local https://api.statsig.com/v3/mcp \
  --header "statsig-api-key: console-YOUR-CONSOLE-API-KEY"
```

Sources: [Amplitude MCP](https://amplitude.com/docs/amplitude-ai/amplitude-mcp), [Statsig MCP with Claude Code](https://docs.statsig.com/integrations/mcp/claude-code). Statsig OAuth only works with Personal Console API keys, which the org owner must enable. Amplitude MCP writes need the "Use MCP (write)" RBAC action; service-account auth is Enterprise-only (OAuth as a user is what a free account gets).

### 8.9 Gotchas

- **Session Record delay.** Raw `[Agent]` events show in Live Events "within about a minute", but the scored `[Agent] Session Record` (signals, task completion) "typically arrives within 15 to 20 minutes" after an explicit close, and after a 30-minute idle timeout plus ~15 minutes otherwise ([quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart), [troubleshooting](https://amplitude.com/docs/amplitude-ai/agent-analytics/troubleshooting)). **Generate demo sessions by ~3:30 PM at the latest and always close sessions explicitly.**
- **Agent Analytics must be enabled on the project** whose API key you use ([quickstart](https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart)). Where the toggle lives: not confirmed — ask at the booth.
- **Define `ai.agent(...)` at module scope**, not per request, or each turn mints a new Agent ID and breaks grouping ([instrument-ai-sdk](https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-ai-sdk)).
- **`patch()` silently drops calls made outside a session** — "the most common cause of 'I instrumented but see no events'" ([SDK reference](https://amplitude.com/docs/sdks/agent-analytics/sdk)).
- **Always `ai.flush()`** before a script exits; call `trackSessionEnd` / `track_session_end` for sessions spanning requests.
- **HTTP path:** `$llm_message` must be an object `{ "text": ... }`; always set `[Agent] Agent ID` (the API returns 200 even when grouping fails); cost is not computed for you; do not invent `[Agent]`-prefixed event names ([HTTP path](https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-http-api)).
- **Empty message strings are dropped** — an unpopulated variable passes most checks but leaves the thread view empty ([verify](https://amplitude.com/docs/amplitude-ai/agent-analytics/verify)).
- **Statsig:** gates are off by default; `initialize()` is a network call — wait for it; call `shutdown()` to flush; in forking servers (Gunicorn/uWSGI) initialize after fork; install name has dashes, import has underscores ([python-core](https://docs.statsig.com/server-core/python-core)).
- **Statsig AI SDK is beta.** Node versions `0.0.1-beta.30`–`.33` fail to import unless you alias `@statsig/statsig-node-core-rc@npm:@statsig/statsig-node-core`; wrap clients **after** `initializeTracing()` in Node ([Node AI SDK](https://docs.statsig.com/agent-experimentation/node)). Included Credits cover playground/judge calls until they run out; then add your own provider key ([overview](https://docs.statsig.com/agent-experimentation/overview)).
- **Experiment statistics will not reach significance in 5.5 hours.** Show the split chart with seeded sessions and say so.
- **Amplitude MCP is not an ingestion path** — events go through the SDK or HTTP API ([MCP docs](https://amplitude.com/docs/amplitude-ai/amplitude-mcp)).

---

## 9. Past hackathons

| Event | Date | Track / prize | Winners |
|---|---|---|---|
| **AI x Product Hackathon** (hosted by Amplitude, SF office; sponsors AWS, New Relic, LanceDB, Fennel.ai) | June 3–4, 2023 | Prizes $2,500–$5,000 per Devpost listing; an "Amplitude Prize" for the project that best uses AI with the Amplitude product ([Devpost](https://ai-x-product-hackathon.devpost.com/), [starter repo](https://github.com/amplitude/ai-product-hackathon)) | Five winning teams named in the recap, without saying which won which prize or how any used Amplitude: **Palate** (article → personal podcast, [devpost](https://devpost.com/software/palate)); **AI-Powered 1:1 Agenda** ([devpost](https://devpost.com/software/1-1-tool)); **Supermark.ai** (AI bookmark manager, [site](https://www.supermark.ai/)); **In Vogue** (personalized fashion outfits, [devpost](https://devpost.com/software/in-vogue)); **NaviGator** (inspection/debugging tool for AI agents, [devpost](https://devpost.com/software/glados-60a7s3)) — [recap](https://amplitude.com/blog/ai-hackathon-amplitude-sf-2023). Which one took the Amplitude Prize: **not found**. |
| **Amplitude London Hackathon** | Nov 2022 (per search result) | £4,000 across 4 challenges ([Devpost](https://amplitude-london-hackathon.devpost.com/)) | **Not found** (page not opened; details from a search summary only). |
| **DubHacks '25** (University of Washington) — Statsig-sponsored | Oct 2025 | "Statsig Hackathon Challenge: AI That Learns What Works" — build an AI-powered product that uses Statsig to experiment, personalize or optimize; judged on creativity, Statsig integration, impact metrics, presentation; prize Nintendo Switch 2 ([Devpost](https://dubhacks25.devpost.com/) — fetched via search summary; direct fetch returned 403) | **Not found.** Statsig congratulated winners on X on Oct 28, 2025 without the indexed text naming them ([X post](https://x.com/statsig/status/1983230012833894852)). A team "Optivise" (AI that runs A/B tests, integrates with Statsig) says it won the separate "DubHacks Next" track — not confirmed as the Statsig track winner ([LinkedIn](https://www.linkedin.com/in/williamdinh/)). |
| **SHIP IT** (Amplitude + Statsig, Left Door, SF) | Oct 28, 2026 | "AI product challenge" — Nintendo Switch 2 or other tech prizes; an evening party rather than a hackathon ([blog](https://amplitude.com/blog/amplitude-github-learning-loop)) | **Not found.** |
| "Amplitude 2017" on HackerEarth | 2017 | Unrelated — a Mastercard-run event that shares the name ([HackerEarth](https://www.hackerearth.com/sprints/amplitude-2017/)) | n/a |

No earlier joint "Best Use of Amplitude or Statsig" track with published winners was found. This appears to be among the first joint developer-prize appearances since the May 2026 partnership — **unverified**.

**Patterns in what wins (limited evidence — stated as inference):**
- The one published Statsig rubric weights **"impact metrics"** and **"Statsig integration"** alongside creativity and presentation. Expect the same instinct here: show a number that moved, not just an SDK call.
- The 2023 Amplitude winners were all **end-user products with a clear consumer use case** (one was a fashion-shopping personalization app; one was an agent debugging tool) — consumer-facing polish counted more than analytics depth.
- The judge is a senior pre-sales leader; a tight demo narrative ("here is the agent, here is the dashboard that tells us variant B converts better and why") matches how SEs present.

---

## 10. Fit with "shopping agent with an audit trail"

The challenge needs (1) reliable context, (2) a meaningful action, (3) visible steps. Agent Analytics' session → turn → tool-call model **is** requirement (3), and a purchase event gives the business outcome Amplitude wants to connect to.

| # | Idea | What you build | Time in a 5.5h build |
|---|---|---|---|
| 1 | **Audit trail as agent events** (shallow) | Wrap the LLM client; one `agent.session` per shopping task with a real `session_id` (order/task ID); wrap every tool (`search_products`, `get_policy`, `compare_prices`, `add_to_cart`, `checkout`) with `tool()` so input/output land on `[Agent] Tool Call`. Put retrieval provenance (e.g. Senso `content_id`, `version_id`, `score`) in the tool output. Show the Amplitude Sessions view turn by turn as the audit trail. | ~30–45 min |
| 2 | **Agent → purchase funnel** | Emit standard product events with the same `user_id`: `Product Recommended`, `Cart Updated`, `Approval Requested`, `Order Placed` (properties: `mode: simulated|test|real`, price, merchant). Build a funnel chart and a dashboard: task → recommendation → approval → order; cost per completed order from `[Agent] Cost USD`. | +30–45 min |
| 3 | **Statsig guardrails and kill switch** | `check_gate("allow_real_checkout")` before any real action (off by default = simulated mode, which the challenge asks you to show clearly); dynamic config `agent_limits` for max spend / require-approval-above; `log_event` for blocked actions. Flip the gate live in the demo. | +30 min |
| 4 | **A/B test the agent with Statsig, read out in Amplitude** (the joint story) | Statsig experiment `shopper_strategy` with a parameter (e.g. `concise` vs `detailed` prompt, or cheap vs strong model, or keyword vs semantic retrieval). Pass it as `experiments=[{key, variant, provider: "statsig"}]`. Seed 20–40 scripted sessions across synthetic users, then show an Amplitude chart of task completion, cost and `Order Placed` split by `[Agent] Experiment Variant`. | +60–75 min (incl. seeding script; start seeding early because of the Session Record delay) |
| 5 | **Trust loop: scores and a custom evaluator** (deep) | Thumbs up/down on each recommendation → `ai.score(name="user-feedback", ...)`. Add a custom Evaluator in the Amplitude UI: "every product claim is backed by a retrieved source" (groundedness) — a direct match to "reliable context". Optionally let the agent read its own history through Amplitude MCP, or use Amplitude/Statsig MCP from Claude Code to build the dashboard and experiment quickly. | +45–90 min; evaluator needs your own model-provider key and results depend on session-close latency |

**Recommended scope for a team also chasing other tracks:** ideas 1 + 2 + 4 (about 2–2.5 hours of one person), with idea 3's single gate if time allows. That uses both sponsors' products in the pattern their own docs describe, and the same events serve as the audit-trail UI for the main challenge.

**Demo script (60 seconds for this sponsor):** run a shopping task → open the Amplitude session and walk the turns/tool calls with sources → show the funnel to `Order Placed` → show the chart split by Statsig variant ("the concise prompt completes more purchases at lower cost per order") → flip the Statsig gate to block real checkout and show the next run stay in simulated mode.

---

## 11. Risks and open questions for the booth

**Risks**
- Session Record / signal scoring lag of 15–20+ minutes could leave the quality columns empty at demo time if sessions are created late.
- New accounts: Agent Analytics may need enabling on the project; custom evaluators need a provider key; MCP Analytics is Early Access only.
- Statsig AI SDK is beta with known install pitfalls; plain gates/experiments via `statsig-python-core` / `@statsig/statsig-node-core` are the safer path.
- A/B results from seeded sessions are illustrative, not statistically valid — say so rather than overclaim.
- Tool-call auto-tracking is not available for Gemini or Mistral wrappers; use `tool()` / `trackToolCall` explicitly.
- If the build uses a model provider the SDK does not wrap (e.g. Arcee), `[Agent] AI Response` must be sent manually with `track_ai_message`, including tokens.

**Questions to ask**
1. Is there a written rubric? Does using **both** Amplitude and Statsig score higher than one used deeply?
2. Which matters more to the judge: depth of Agent Analytics instrumentation, or the experiment readout?
3. How do we enable Agent Analytics on a fresh free project, and is there a way to speed up Session Record enrichment for the demo?
4. Are custom evaluators usable on a free org today without our own provider key (any included credits)?
5. Is there a first-party Statsig → Amplitude link (exposures flowing into Amplitude), or is passing `provider: "statsig"` through the AI SDK the intended integration?
6. Can we get MCP Analytics early access today if we build our own MCP server for the shopping tools?
7. Does Amplitude render tool outputs/artifacts (product cards, receipts) in the session view — "Render agent artifacts" is referenced in the SDK docs?
8. Is the prize cash, and is it per team?
9. Do Sid Nayak's and Jackson Yang's talks include a recommended starter repo or template?

---

## 12. Sources

Event
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack

Relationship / company
- https://www.amplitude.com/blog/amplitude-and-statsig-partnership
- https://martech.org/amplitude-and-statsig-deal-raises-questions-for-customers/
- https://www.convert.com/blog/a-b-testing/statsig-moves-to-amplitude/
- https://vwo.com/blog/amplitude-statsig-partnership/ (search result only)
- https://longbridge.com/news/286149957.md (search result only)
- https://siliconangle.com/?p=709797 (search result only)
- https://amplitude.com/press/third-quarter-2026-financial-results
- https://investors.amplitude.com/news-releases/news-release-details/amplitude-introduces-agentic-ai-analytics-next-era-product (search result only)
- https://amplitude.com/blog/amplitude-github-learning-loop
- https://en.wikipedia.org/wiki/Amplitude,_Inc. (search result only)
- https://www.ivp.com/content/amplitude-making-an-increasingly-digital-first-world-a-better-place/ (search result only)
- https://news.crunchbase.com/news/amplitude-public-direct-listing-analytics-platform (search result only)
- https://venturebeat.com/business/statsig-secures-100m-series-c-funding-to-transform-product-development-valuing-startup-at-1-1-billion/ (search result only)
- https://dc.fortune.com/2025/05/06/statsig-series-c-100-million-1-1-billion-eppo-datadog (search result only)
- https://amplitude.com/pricing
- https://www.statsig.com/pricing
- https://amplitude.com/customers
- https://www.statsig.com/customers

People
- https://amplitude.com/blog/2025-founders-awards
- https://amplitude.com/blog/meet-the-ampliteer-bethany-mangan
- https://village.ai/person/sid-nayak-NTExODY0ODEw (search result only)
- https://theorg.com/org/amplitude/org-chart/sid-nayak (search result only)

Amplitude docs
- https://amplitude.com/docs
- https://amplitude.com/docs/llms.txt
- https://amplitude.com/docs/amplitude-ai/agent-analytics/overview
- https://amplitude.com/docs/amplitude-ai/agent-analytics/quickstart
- https://amplitude.com/docs/amplitude-ai/agent-analytics/setup
- https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-ai-sdk
- https://amplitude.com/docs/amplitude-ai/agent-analytics/instrument-http-api
- https://amplitude.com/docs/amplitude-ai/agent-analytics/verify
- https://amplitude.com/docs/amplitude-ai/agent-analytics/troubleshooting
- https://amplitude.com/docs/amplitude-ai/agent-analytics/taxonomy
- https://amplitude.com/docs/amplitude-ai/agent-analytics/track-experiments
- https://amplitude.com/docs/amplitude-ai/agent-analytics/run-an-agent-experiment
- https://amplitude.com/docs/sdks/agent-analytics/sdk
- https://amplitude.com/docs/amplitude-ai/amplitude-mcp
- https://amplitude.com/docs/amplitude-ai/mcp-analytics
- https://amplitude.com/docs/get-started/install-server-sdk
- https://amplitude.com/docs/sdks/analytics/python/python-sdk
- https://amplitude.com/docs/feature-experiment/experiment-quick-start

Statsig docs
- https://docs.statsig.com/welcome
- https://docs.statsig.com/llms.txt
- https://docs.statsig.com/agent-experimentation/overview
- https://docs.statsig.com/agent-experimentation/python
- https://docs.statsig.com/agent-experimentation/node
- https://docs.statsig.com/server-core/python-core
- https://docs.statsig.com/server-core/node-core
- https://docs.statsig.com/integrations/mcp/overview
- https://docs.statsig.com/integrations/mcp/claude-code
- https://docs.statsig.com/integrations/openai (index entry only)

Past hackathons
- https://amplitude.com/blog/ai-hackathon-amplitude-sf-2023
- https://ai-x-product-hackathon.devpost.com/ (search result only)
- https://github.com/amplitude/ai-product-hackathon (search result only)
- https://amplitude-london-hackathon.devpost.com/ (search result only)
- https://dubhacks25.devpost.com/ (search result only; direct fetch 403)
- https://x.com/statsig/status/1983230012833894852 (search result only)
- https://www.linkedin.com/in/williamdinh/ (search result only)
- https://www.hackerearth.com/sprints/amplitude-2017/ (search result only)
