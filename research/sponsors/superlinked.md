# Superlinked — SIE / SIE Serverless (sponsor research)

> Researched 2026-11-19 for the Personal Agents Hackathon (2026-11-20, SF). Challenge: "build a shopping agent with an audit trail".
> Method: official docs pulled as raw Markdown (`superlinked.com/docs/*.md`, `llms-full.txt`), the `superlinked/cookbook` repo cloned and read (HEAD commit 2026-11-16), plus blog, GitHub and web search. Anything I could not confirm is marked **unverified** or **not found**.

---

## 1. TL;DR

- **What it is.** SIE = **Superlinked Inference Engine**: an Apache-2.0 inference server for *small, task-specific open models* (embeddings, rerankers, extractors/classifiers, OCR, speech-to-text, guardrails, PII redaction, small/mid LLMs) behind four primitives — `encode`, `score`, `extract`, `generate` — plus OpenAI-compatible endpoints. **SIE Serverless** is the hosted pay-per-call version at `https://api.superlinked.com` (launched 2026-10-16). ([docs](https://superlinked.com/docs), [launch post](https://superlinked.com/blog/launch), [llms.txt](https://superlinked.com/llms.txt))
- **What the prize wants (inferred).** Two tracks, each $500 cash + $500 credits. (a) *Best Personal Agent Harness Using SIE Serverless* = the best agent scaffold (loop + tools + checks + approval) whose tools run on the hosted API. (b) *Best Technical Build for Personal Agents with SIE* = deepest engineering with SIE in any deployment (several primitives, routing, measured cost/latency, MCP). Neither track has published criteria — **confirm at the booth**. ([event page](https://tokensand.com/personalagenthack))
- **The sponsor's house style is our challenge.** Their cookbook pattern is: code handles facts, a small model reads only what code cannot, Python verifies every quote/number, a person approves, and every output carries its source. That is "shopping agent with an audit trail" almost word for word. ([cookbook README](https://github.com/superlinked/cookbook), [workflow-vs-agent post](https://superlinked.com/blog/agentic-vs-workflows-small-models))
- **Fastest qualifying integration (~30 min).** Sign up at `console.superlinked.com` (no card, $20 credit), `pip install sie-sdk`, then call `client.score("Qwen/Qwen3-Reranker-4B", query, items)` to rerank product candidates after hard filters, and log `item_id/score/rank` plus `served_by` into the audit trail. Copy from `ecommerce/rerank_search_within_filters`. ([serverless page](https://superlinked.com/serverless), [quickstart](https://superlinked.com/docs/quickstart))
- **Best way to win.** Use **3–5 different SIE models as agent tools** (Decide for routing/yes-no gates, GLiNER2 for field extraction, reranker for retrieval, OCR for receipts/screenshots, pii-redact or Qwen3Guard as a guard), keep a frontier model only for planning, and show a per-step ledger: model, latency, cost, evidence span, check result. Warm the models before the demo — cold starts can take minutes.

---

## 2. What they do

### Product
SIE is described in the docs as "an open-source inference server for AI models" that "runs encoders, rerankers, extractors, and generation models", "serving 150+ models through four simple API primitives" ([docs](https://superlinked.com/docs)). `llms.txt` glosses it as a "Small-model Inference Engine" ([llms.txt](https://superlinked.com/llms.txt)); the official expansion everywhere else is **Superlinked Inference Engine**. Model counts vary by page (85+, 100+, 150+, 200+) — the catalog is growing; Serverless serves a subset ("46 models on Serverless", per the [serverless page](https://superlinked.com/serverless)).

Three ways to run the same engine ([choosing post](https://superlinked.com/blog/choosing-where-small-models-run), [serverless page](https://superlinked.com/serverless)):

| Option | What it is |
|---|---|
| **SIE Serverless** | Superlinked runs the GPUs; API key; pay per call. SOC 2 Type 2, US or EU residency. |
| **SIE Cluster** / "Native Inference" | Helm/Terraform in your own cloud (AWS, GCP, Azure, Alibaba); Native Inference adds Superlinked engineers. |
| **SIE Local** | `pip install "sie-server[local]" && sie-server serve` on an NVIDIA GPU or Apple Silicon. |

Client code is identical across the three; only `SIE_BASE_URL` / `SIE_API_KEY` change ([cookbook `.env.example`](https://github.com/superlinked/cookbook)).

### Primitives and endpoints ([HTTP API reference](https://superlinked.com/docs/reference/api))

| Primitive | Native endpoint | Tasks |
|---|---|---|
| Encode | `POST /v1/encode/:model` | dense, sparse, multivector (ColBERT), image/multimodal embeddings |
| Score | `POST /v1/score/:model` | cross-encoder reranking, request routing |
| Extract | `POST /v1/extract/:model` | NER, schema field extraction, classification / typed decisions, relations, OCR, doc-to-Markdown, object detection, speech-to-text, PII |
| Generate | `POST /v1/generate/:model` (preview, no streaming) | text generation |

OpenAI-compatible: `/v1/embeddings`, `/v1/chat/completions` (with tool calling and `response_format` JSON schema), `/v1/completions`, `/v1/responses`, `/v1/audio/transcriptions`, `/v1/rerank`, and `/v1/decisions` ([API reference](https://superlinked.com/docs/reference/api), [tool calling](https://superlinked.com/docs/generate/tool-calling), [OpenAI Decisions migration](https://superlinked.com/docs/migrate/openai-decisions)). It also serves the **Anthropic Messages API at `POST /v1/messages`**, translated onto chat completions, with runnable examples for Claude Code and OpenCode ([tool-calling doc](https://superlinked.com/docs/generate/tool-calling)). Native wire format defaults to msgpack; send `Accept: application/json` for JSON ([API reference](https://superlinked.com/docs/reference/api)).

Technical differentiators claimed: many models share one GPU with on-demand loading and LRU eviction; one gateway/queue for all models; scale to zero; per-model quality/latency targets checked in CI ([docs](https://superlinked.com/docs)).

### Positioning
- Launch headline: "Most AI tasks no longer need a frontier model" — vendor-reported: across 14 task tests, open models on SIE reached a median 98% of a frontier model's score at 3% of its cost ([launch post](https://superlinked.com/blog/launch)).
- Tagline at this event: "Task-specific inference for AI agents" ([event page](https://tokensand.com/personalagenthack)). GitHub: "Self-hosted inference for agents. Every open model your agents call, served from one cluster in your cloud." ([sie repo](https://github.com/superlinked/sie))
- CEO thesis: "carve off tasks, not models" — define a task's input/output, move it to a specialist model, judge by cost per task ([post](https://superlinked.com/blog/carve-off-tasks-not-models)).

### How it differs from the old Superlinked framework
The original `superlinked/superlinked` repo (a Python framework for multi-attribute vector search/recommendation with a self-hosted REST server) was **archived on 2026-05-29**; its README says "Superlinked has evolved and this repo is deprecated" and points to SIE ([repo](https://github.com/superlinked/superlinked)). VectorHub is also marked deprecated ([org page](https://github.com/superlinked)). The old framework was a retrieval/query layer over vector DBs; SIE is the model-serving layer. Do **not** use old `superlinked` Spaces/Index/Query code — it is not what the prize is about. (One docs FAQ still calls SIE "the inference layer underneath" the framework — stale wording.)

### Competitors (as they frame them)
Self-host peers: vLLM, SGLang, HuggingFace TEI, Infinity, FastEmbed, Triton, KServe, llm-d, NVIDIA Dynamo. Hosted peers: OpenAI, Cohere, Voyage, Modal, Amazon Rekognition/Textract, Azure Document Intelligence. Sources: their comparison posts and migration guides ([blog index](https://superlinked.com/blog), [docs nav](https://superlinked.com/docs)).

---

## 3. Company facts

| Fact | Detail | Source |
|---|---|---|
| Founded | 2021 | [about](https://superlinked.com/about) |
| Founders | Daniel Svonava (CEO; ex ML Tech Lead, YouTube Ads at Google) and Ben Gutkovich (COO; described as ex-McKinsey by third parties) | [about](https://superlinked.com/about), [theorg](https://theorg.com/org/superlinked/org-chart/daniel-svonava) |
| HQ / offices | San Francisco; team also in London, Budapest, Tel Aviv | [about](https://superlinked.com/about) |
| Funding | "More than $12M" per their site. $9.5M seed led by Index Ventures (March 2024) with Theory Ventures, 20SALES, Firestreak. Third-party totals differ ($10.15M–$10.8M). No 2026 round found. | [about](https://superlinked.com/about), [nordic9](https://nordic9.com/news/superlinked-raised-a-95-million-seed-round-led-by-index-ventures-with-additional-investors-including-theory-ventures-20sales-and-firestreak), [CB Insights](https://www.cbinsights.com/company/superlinked/financials) |
| Investors listed | Index Ventures, Theory Ventures, Sequoia Scout, Samsung Next, Firestreak, ELC, Kaya VC, HOF Capital, SAV, 20SALES, Sixty Degree Capital, MMC Ventures, Episode 1, Concept Ventures, Vectr, Embedding VC, Ferocity Capital | [about](https://superlinked.com/about) |
| Size | ~32 people — **unverified** (single third-party source) | [yespress](https://yespress.io/superlinked-inc) |
| GitHub | `superlinked/sie` ~3.5K stars, 339 forks; latest release v0.11.1 (2026-11-12) | [sie repo](https://github.com/superlinked/sie), GitHub releases API |

Timeline:
- 2026-05-29 — old framework repo archived ([repo](https://github.com/superlinked/superlinked)).
- 2026-08-14 — Superlinked x Qwen hackathon, London ([recap](https://superlinked.com/blog/qwen-hackathon-london)).
- 2026-10-16 — SIE launch post; Serverless with $20 free credit, "apply for $500 more" ([launch post](https://superlinked.com/blog/launch)).
- 2026-11-05 to 11-13 — burst of ~15 posts by Rahul Parundekar on small models per industry, agents vs workflows, small models as agent tools ([blog](https://superlinked.com/blog)).
- 2026-11-16 — cookbook last commit before the event (201 commits, 3 stars — it is brand new).
- 2026-12-05 to 12-20 — "Infer-a-thon with Superlinked" virtual hackathon with Geek Room ([Luma](https://luma.com/f7ef30j2)).

---

## 4. Customers & use cases

- **Named paying SIE customers: not found.** The about page, launch post and solution pages name none ([about](https://superlinked.com/about), [launch](https://superlinked.com/blog/launch)).
- **Fellow** (fellow.legal, an AI-native law firm) appears in a conversation post with the CEO, but the post says they are *exploring* self-hosted models, not that they deploy SIE ([post](https://superlinked.com/blog/building-legal-ai-agents-with-small-models)).
- Their posts cite third parties as evidence for the small-model thesis — Shopify, Instacart, Etsy, eBay, Target, Cloudflare, Sword Health, Marsh McLennan, LexisNexis, Harvey, Datadog and others. These are **industry examples, not Superlinked customers** ([post](https://superlinked.com/blog/small-open-models-across-industries)).
- Verticals with solution pages and cookbook folders: legal, finance, insurance, healthcare, manufacturing, **e-commerce**, **retail**, security, SRE ([serverless page](https://superlinked.com/serverless), [cookbook](https://github.com/superlinked/cookbook)).
- E-commerce pitch: "Inference for faster, cheaper & smarter e-commerce agents"; vendor-reported latency e.g. classify 13.1 ms, rerank 55 ms, extract entities 24 ms; illustrative monthly cost $61,330 → $3,870 ([e-commerce page](https://superlinked.com/e-commerce)).

---

## 5. Why they are at this hackathon

Inferred from launches, DevRel output and prize wording:

1. **Drive Serverless signups one month after launch.** Track (a) names "SIE Serverless" explicitly and the prize is half credits. The $20 no-card credit removes signup friction ([launch post](https://superlinked.com/blog/launch)).
2. **Reposition from "vector search framework" to "inference for agents".** Every recent page says "agents". They want public repos showing an agent whose narrow steps run on SIE models.
3. **Seed the "personal agents" category.** The cookbook's `personal/` folder has only two recipes, and its README invites the pattern for "tracking return windows from order confirmations" ([recipe README](https://github.com/superlinked/cookbook/tree/main/personal/turn_messages_into_actions)). A strong shopping agent is a plausible future cookbook entry or blog post — their London recap wrote up all nine teams with repo links ([recap](https://superlinked.com/blog/qwen-hackathon-london)).
4. **Prove the hybrid thesis.** Large model plans; small models do reading, matching, masking, deciding; code checks. They want to see the large model used only where it changes the result ([agent-tools post](https://superlinked.com/blog/small-open-models-as-agent-tools)).
5. **Collect cost/latency evidence.** Their content is full of measured numbers and honest misses. Projects that report numbers fit their voice.

---

## 6. Prize track

**Exact wording** ([event page](https://tokensand.com/personalagenthack)):
- Tagline: "Task-specific inference for AI agents."
- **Best Personal Agent Harness Using SIE Serverless** — $500 cash + $500 in SIE Serverless credits.
- **Best Technical Build for Personal Agents with SIE** — $500 cash + $500 in SIE Serverless credits.
- Links given: serverless page, docs, cookbook. No descriptions or criteria published. The Luma page lists no extra detail ([Luma](https://luma.com/personalagenthack)).

**What each likely rewards (my inference, not sponsor-stated):**

| Track | Likely reading | What to show |
|---|---|---|
| Harness (Serverless) | The agent *scaffold*: loop, tool definitions, guards, verification, approval gate, trace — with tools backed by the hosted API. "Harness" does not appear anywhere in Superlinked's docs or cookbook (searched; **not found**), so they are using the general industry term. | Calls to `api.superlinked.com`; small models exposed as tools (function tools or an MCP server); bounded loop; checks in code; human approval; a visible trace. A reusable harness others could fork scores best. |
| Technical build (SIE) | Says "SIE", not "Serverless" → any deployment. Rewards depth. | Several primitives and models; routing cheap-vs-strong; measured latency/cost vs an all-frontier baseline; threshold handling; cold-start handling; maybe SIE Local fallback or the `/v1/messages` endpoint. |

One project can plausibly be entered for both — **ask whether a team can win both**.

### People

**Daniel Svonava — Founder & CEO, judge and speaker.**
- Background: ex ML Tech Lead for YouTube Ads at Google ([about](https://superlinked.com/about)); earlier IBM Research Zurich per a third-party profile (**unverified**, [yespress](https://yespress.io/daniel-svonava)).
- Talks/writing: launch post "Most AI tasks no longer need a frontier model" ([post](https://superlinked.com/blog/launch)); "Should You Self-Host Inference?" ([blog](https://superlinked.com/blog)); AI Engineer talk "Large clusters for small models" (listing dated 2026-09-19; contents **unverified**, [podwise](https://podwise.ai/episodes/8970193)); Joe Reis Show conversation summarised in "Carve Off Tasks, Not Models" ([post](https://superlinked.com/blog/carve-off-tasks-not-models)).
- What he says he values: "We don't think about it as okay which open source model, but more like how do we carve off and define a task." and "Just follow the money." — i.e. task definition first, cost per task over price per token, sub-agents as the expensive part worth moving to a cheaper tier ([post](https://superlinked.com/blog/carve-off-tasks-not-models)).
- His talk title/time at this event: **not found**.

**Rahul Parundekar — listed as DevRel @ Superlinked, judge.**
- Blog byline reads "in-residence at Superlinked" ([post](https://superlinked.com/blog/agentic-vs-workflows-small-models)). He authored ~15 posts in Nov 2026 and, judging by matching content, most likely the cookbook's framing (authorship of the repo itself **unverified**).
- Prior background: founder of AI Hero, MS CS from USC (2010), organises MLOps events in SF — from a single MLOps Community speaker profile, may be dated ([profile](https://home.mlops.community/public/videos/declarative-mlops-streamlining-model-serving-on-kubernetes)). LinkedIn not checked.
- What he says he likes to see (his own posts):
  - "In a workflow, your code picks each step and calls a model where a step needs one" vs "in an agent, a model picks its next step in a loop". "Cost is why I lean toward the workflow whenever the steps allow it." ([post](https://superlinked.com/blog/agentic-vs-workflows-small-models))
  - Agents need: bounded loops (his example uses a 10-tool-call budget), small models as tools, code that checks every quote word-for-word against its source, one bounded retry, a person making the final call ([same post](https://superlinked.com/blog/agentic-vs-workflows-small-models)).
  - Each step goes to the cheapest owner: Python for rules and sums, a small model for reading/matching, the large model for choosing the next step, code plus a person for sign-off. "The large model never sees the raw complaint." ([agent-tools post](https://superlinked.com/blog/small-open-models-as-agent-tools))
  - E-commerce rule: "a model's guess never overrules a fact the rules can check in the PIM or the stock system." ([e-commerce post](https://superlinked.com/blog/ecommerce-search-and-catalog-with-small-models))
  - He reports misses honestly (e.g. 8 of 10 cases passed; pii-redact masked a meter reading as a phone number). Showing a failure case and how the check caught it will land well.

---

## 7. What they expect to see used (ranked by likely impact)

1. **Several task models used as agent tools, each on the right job** — the core thesis. Minimum credible set for shopping: reranker + Decide + GLiNER2 extraction. Their own Infer-a-thon rule says SIE must provide "at least one supporting capability, not just a single embeddings call" ([Luma](https://luma.com/f7ef30j2)).
2. **`fastino/GLiNER2.5-Decide` typed decisions** (yes/no, pick-one, score with probabilities) for routing and gates such as "does this listing meet the constraint?" or "is this action a purchase needing approval?". Featured in 9 recipes ([MODELS.md](https://github.com/superlinked/cookbook/blob/main/MODELS.md)).
3. **`Qwen/Qwen3-Reranker-4B` after hard filters** — their flagship e-commerce recipe ([recipe](https://github.com/superlinked/cookbook/tree/main/ecommerce/rerank_search_within_filters)).
4. **`fastino/gliner2-large-v1` schema extraction** with values verified as spans of the source text ([personal recipe](https://github.com/superlinked/cookbook/tree/main/personal/turn_messages_into_actions)).
5. **Guard steps on small models**: `superlinked/pii-redact` before anything reaches the big LLM; `Qwen/Qwen3Guard-Gen-4B` for prompt-injection/unsafe content from scraped pages ([guarded-agent recipe](https://github.com/superlinked/cookbook/tree/main/agentic/fix_laptop_compliance_with_guarded_agent)).
6. **Vision/OCR**: `lightonai/LightOnOCR-3-4B` or `zai-org/GLM-OCR` for receipts, order confirmations, product photos; `google/siglip-so400m-patch14-384` for photo+text product search ([MODELS.md](https://github.com/superlinked/cookbook/blob/main/MODELS.md)).
7. **Open LLM on SIE for the cheap lane** via `/v1/chat/completions` with tool calling or JSON-schema output (`Qwen/Qwen3.8-27B-FP8`, `Qwen/Qwen3.5-4B`) and a router that escalates to a frontier model only when needed — the pattern that won their London hackathon ([recap](https://superlinked.com/blog/qwen-hackathon-london)).
8. **MCP server wrapping SIE tools** so any harness (Claude Code etc.) can use them, with policy enforced server-side ([MCP recipe](https://github.com/superlinked/cookbook/tree/main/agentic/screen_applicants_through_mcp_server), [SIE MCP post](https://superlinked.com/blog/serve-private-document-tools-sie-mcp)).
9. **Provenance from SIE itself**: `result["request"]["served_by" | "upstream" | "fallback_reason"]`, item ids echoed in scores, entity `start`/`end` offsets and `score` — all natural audit-trail fields ([SIE Relay doc](https://superlinked.com/docs/cloud/sie-relay), [API reference](https://superlinked.com/docs/reference/api)).
10. **Config in `rules.yaml`** (models, thresholds, policies) and a `runs/<timestamp>/` output folder — mirrors cookbook conventions ([AGENTS.md](https://github.com/superlinked/cookbook/blob/main/AGENTS.md)).

Low impact: a single `/v1/embeddings` call through the OpenAI SDK. It qualifies technically but is exactly what they say is not enough.

---

## 8. Quickstart

### Signup and auth
- Console: `https://console.superlinked.com` — keys at `https://console.superlinked.com/console/keys` ([cookbook README](https://github.com/superlinked/cookbook)).
- New accounts: **$20 free credit, no credit card**; "apply for $500 more" launch credits ([launch post](https://superlinked.com/blog/launch), [serverless page](https://superlinked.com/serverless)). Whether hackathon attendees get extra credits: **not found — ask**.
- Base URL `https://api.superlinked.com`; Bearer key with prefix `sk-sie-…` ([serverless page](https://superlinked.com/serverless), [OpenAI Decisions doc](https://superlinked.com/docs/migrate/openai-decisions)).

### Install
```bash
pip install sie-sdk                 # Python (cookbook pins sie-sdk>=0.11.1, Python 3.12)
npm install @superlinked/sie-sdk    # TypeScript
```
Source: [quickstart](https://superlinked.com/docs/quickstart), [cookbook pyproject](https://github.com/superlinked/cookbook/blob/main/personal/turn_messages_into_actions/pyproject.toml).

### Run a cookbook recipe as a smoke test
```bash
git clone https://github.com/superlinked/cookbook && cd cookbook
cp .env.example .env   # then set SIE_API_KEY; every recipe reads this root .env
cd ecommerce/rerank_search_within_filters
uv run recipe.py
```
Source: [cookbook README](https://github.com/superlinked/cookbook). Agentic recipes also need `ANTHROPIC_API_KEY`.

### Client (Serverless) — from the docs
```python
import os
from sie_sdk import SIEClient
from sie_sdk.types import Item

client = SIEClient("https://api.superlinked.com", api_key=os.environ["SIE_API_KEY"])
```
Source: [SIE Relay doc](https://superlinked.com/docs/cloud/sie-relay). The cookbook uses `SIEClient(api_key=..., base_url=..., timeout_s=900)`.

### Encode (embeddings)
```python
result = client.encode("BAAI/bge-m3", Item(text="Hello world"))
print(result["dense"].shape)
results = client.encode("BAAI/bge-m3", [Item(text="First document"), Item(text="Second document")])
```
Adapted from the [quickstart](https://superlinked.com/docs/quickstart) (model swapped to one on Serverless). The cookbook passes `is_query=True` and `instruction=...` for queries with `Qwen/Qwen3-Embedding-4B`.

### Score (rerank) — response shape from the docs
```python
query = Item(id="query", text="waterproof trail running shoes under $120")
items = [Item(id="sku-1", text="..."), Item(id="sku-2", text="...")]
result = client.score("Qwen/Qwen3-Reranker-4B", query, items)
for entry in result["scores"]:           # sorted, rank 0 = most relevant
    print(entry["item_id"], entry["rank"], entry["score"])
```
Shape `{"model", "scores": [{"item_id", "score", "rank"}], "usage"}` per the [API reference](https://superlinked.com/docs/reference/api). `instruction=` is supported ([rerank recipe](https://github.com/superlinked/cookbook/tree/main/ecommerce/rerank_search_within_filters)).

### Extract — entities
```python
result = client.extract(
    "urchade/gliner_multi-v2.1",
    Item(text="Tim Cook is the CEO of Apple."),
    labels=["person", "organization"],
)
for entity in result["entities"]:
    print(entity["label"], entity["text"])   # also score, start, end
```
Source: [quickstart](https://superlinked.com/docs/quickstart), [API reference](https://superlinked.com/docs/reference/api).

### Extract — schema fields (GLiNER2), as the cookbook does it
```python
schema = {"type": "object", "properties": {
    "amount":   {"type": "string", "description": "amount of money due, such as $84.17"},
    "due_date": {"type": "string", "description": "date by which a payment is due"},
}}
reply = client.extract("fastino/gliner2-large-v1", Item(id="m1", text=text),
                       output_schema=schema, options={"overflow_policy": "error"})
fields = reply["data"]        # dict of field -> string; verify each value is in `text`
```
Adapted from [`turn_messages_into_actions/recipe.py`](https://github.com/superlinked/cookbook/blob/main/personal/turn_messages_into_actions/recipe.py). Model context ~512 tokens; the recipe windows text at ~180 words.

### Extract — typed decision (GLiNER2.5-Decide), as the cookbook does it
```python
schema = {"action": {
    "type": "choice",
    "instructions": "What should the person do about this message?",
    "criteria": {"pay": "a bill, invoice or payment that is due",
                 "fyi": "a newsletter, promotion, delivery notice or receipt with nothing to do"},
}}
reply = client.extract("fastino/GLiNER2.5-Decide", Item(id="m1", text=text), output_schema=schema)
probs = reply["data"]["action"]["probabilities"]   # {"pay": 0.93, "fyi": 0.07}
```
Adapted from the same recipe (`type_schema`, `read_type_probabilities`). The recipe asks the person when top probability < 0.50 or the top-two margin < 0.15.

Same model through the OpenAI SDK ([doc](https://superlinked.com/docs/migrate/openai-decisions)):
```python
from openai import OpenAI
client = OpenAI(api_key="sk-sie-…", base_url="https://api.superlinked.com/v1")
decision = client.decisions.create(
    model="fastino/GLiNER2.5-Decide",
    input="I was charged twice. Please refund one charge today.",
    questions=[
        {"type": "predicate", "name": "refund", "instructions": "Does the customer request a refund?"},
        {"type": "choice", "name": "team", "instructions": "Which team should handle this?",
         "choices": [{"value": "billing"}, {"value": "technical"}]},
    ],
)
refund, team = decision.answers
print(refund.probability, team.choice)
```
Requires an `openai` SDK version that has `client.decisions` — check your installed version.

### Extract — OCR and speech (cookbook shapes)
```python
reply = client.extract("lightonai/LightOnOCR-3-4B", Item(id="ocr/1", images=[png_bytes]))
markdown = next(e["text"] for e in reply["entities"] if e["label"] == "markdown")

audio = {"data": path.read_bytes(), "format": "m4a"}
reply = client.extract("nvidia/parakeet-tdt-0.6b-v3", Item(id="t/1", audio=audio),
                       options={"timestamp_granularities": ["segment"]})
text = " ".join(s["text"] for s in reply["data"]["segments"])
```
Adapted from [`turn_messages_into_actions/recipe.py`](https://github.com/superlinked/cookbook/blob/main/personal/turn_messages_into_actions/recipe.py). Recipe downsizes images to ≤2048 px on the long edge.

### Chat, tool calling, structured output (OpenAI-compatible)
```bash
curl -i https://api.superlinked.com/v1/chat/completions \
  -H "Authorization: Bearer $SIE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"Qwen/Qwen3.5-4B","messages":[{"role":"user","content":"hello"}],"max_completion_tokens":16}'
```
Source: [SIE Relay doc](https://superlinked.com/docs/cloud/sie-relay) (minus the opt-out header).

```python
from openai import OpenAI
client = OpenAI(base_url="https://api.superlinked.com/v1", api_key=os.environ["SIE_API_KEY"])
response = client.chat.completions.create(
    model="Qwen/Qwen3.5-4B",
    messages=[{"role": "user", "content": "What's the weather in Paris right now?"}],
    tools=tools, max_tokens=512, temperature=0,
)
```
Adapted from the [tool-calling doc](https://superlinked.com/docs/generate/tool-calling) (docs use `http://localhost:8080/v1`). Standard OpenAI tool loop: `finish_reason == "tool_calls"`, arguments are a JSON string, reply with `role: "tool"` messages. Docs advise `temperature=0` and a round cap (5). The SDK also has `client.chat_completions(model, messages, max_completion_tokens=..., response_format={"type":"json_schema", ...})` ([keep-facts recipe](https://github.com/superlinked/cookbook/blob/main/personal/keep_facts_current_from_notes/recipe.py)). A forced `tool_choice` cannot be combined with `response_format`.

### Native HTTP
```bash
curl -X POST https://api.superlinked.com/v1/encode/BAAI/bge-m3 \
  -H "Authorization: Bearer $SIE_API_KEY" \
  -H "Content-Type: application/json" -H "Accept: application/json" \
  -d '{"items": [{"text": "hello"}]}'
```
Composed from the [retries doc](https://superlinked.com/docs/cloud/retries) and [API reference](https://superlinked.com/docs/reference/api).

### Serverless model IDs and prices (from the [model catalog](https://superlinked.com/models); verify in console)

| Task | Model ID | Listed price |
|---|---|---|
| Embeddings | `Qwen/Qwen3-Embedding-4B` | $0.06 / 1M tokens |
| Embeddings (dense+sparse) | `BAAI/bge-m3` | ~$0.04 / 1M tokens |
| Rerank | `Qwen/Qwen3-Reranker-4B` | $0.0425 / 1M tokens |
| Typed decisions | `fastino/GLiNER2.5-Decide` | $0.28 / 1M tokens |
| Field extraction | `fastino/gliner2-large-v1` | $0.186 / 1M tokens |
| PII | `superlinked/pii-redact` | $0.08 / 1M tokens |
| Guardrail | `Qwen/Qwen3Guard-Gen-4B` | $0.05 in / $0.75 out per 1M |
| OCR | `lightonai/LightOnOCR-3-4B` | $0.14 in / $1.37 out per 1M |
| OCR | `zai-org/GLM-OCR` | $0.25 / 1,000 images |
| Speech | `nvidia/parakeet-tdt-0.6b-v3` | $0.00075 / audio minute |
| Image+text embed | `google/siglip-so400m-patch14-384` | $0.0233 / 1,000 images |
| LLM | `Qwen/Qwen3.8-27B-FP8` | $0.25 in / $2.75 out per 1M |
| LLM (small) | `Qwen/Qwen3.5-4B` | $0.305 in / $1.53 out per 1M |
| LLM | `zai-org/GLM-5.3-Flash` | $0.15 in / $0.50 out per 1M |

$20 is ample for a hackathon at these prices.

### Rate limits ([doc](https://superlinked.com/docs/cloud/rate-limits))

| Endpoints | Per key | Per org |
|---|---|---|
| `/v1/encode`, `/v1/score`, `/v1/extract`, `/v1/embeddings`, `/v1/audio/transcriptions` | 50 req/s, 32 in flight | 200 req/s, 128 in flight |
| `/v1/generate`, `/v1/chat/completions`, `/v1/completions`, `/v1/responses`, `/v1/messages` | **5 req/s, 4 in flight** | 25 req/s, 20 in flight |

Over-limit returns `429` with `Retry-After`; nothing is billed; SDKs retry automatically. Batch many items into one call — one call counts as one request.

### Gotchas
- **Cold starts are the main demo risk.** Capacity scales to zero. The cookbook warns "The first call can wait minutes while SIE Serverless starts a cold model" and sets a 900 s wait ([AGENTS.md](https://github.com/superlinked/cookbook/blob/main/AGENTS.md)). Un-relayed requests get `503 PROVISIONING` + `Retry-After`; SDKs wait by default (`wait_for_capacity`, `provision_timeout_s` default 900) ([SIE Relay doc](https://superlinked.com/docs/cloud/sie-relay)). **Embedding and reranking are never relayed**, so they always wait for dedicated capacity. Warm every model a few minutes before the demo and keep a keep-alive ping running.
- **SIE Relay**: generation/extraction may be served by a partner while capacity starts; same price and format, output can differ slightly. Check `result["request"]["served_by"]` (SDK ≥ 0.10.0) ([doc](https://superlinked.com/docs/cloud/sie-relay)).
- **Generation concurrency is 4 in flight per key** — do not fan out parallel LLM tool calls.
- **Thresholds are model-specific.** Swapping reranker/embedding models moves score distributions; the 0.6B reranker "crowds near 1.0" ([MODELS.md](https://github.com/superlinked/cookbook/blob/main/MODELS.md)).
- **GLiNER2 empty result** can come back as HTTP 500 on Serverless; the recipe treats it as "no fields found" ([recipe.py comment](https://github.com/superlinked/cookbook/blob/main/personal/turn_messages_into_actions/recipe.py)).
- **Short context on extract models** (~512 tokens for GLiNER/Decide/gliclass) — window long text; `INPUT_TOO_LONG` is a 400 ([API reference](https://superlinked.com/docs/reference/api)).
- **Local on Apple Silicon** needs Python 3.12 and the `[local]` extra; generation runs as a separate MLX process on port 8081 ([quickstart](https://superlinked.com/docs/quickstart)). Not worth the time in a 5.5 h build unless it is the point of the demo.
- Small models at `tool_choice: auto` sometimes answer instead of calling a tool; use `temperature=0` ([tool-calling doc](https://superlinked.com/docs/generate/tool-calling)).
- The docs' quickstart targets `localhost:8080`; for Serverless swap the base URL and add the key. `/v1/generate` is a preview surface.
- The **SIE MCP edge** (`packages/sie_mcp`, 8 document tools) is documented against a cluster endpoint; whether it works against Serverless is **unverified** ([post](https://superlinked.com/blog/serve-private-document-tools-sie-mcp)).

---

## 9. Past hackathons

### Superlinked x Qwen Hackathon — London, 2026-08-14
- Hosted at Alibaba Cloud's London office; invite-only, ~25 seats; nine teams ([recap](https://superlinked.com/blog/qwen-hackathon-london), [Hackathon Radar](https://www.hackathonradar.com/database/hackathon/superlinked-x-qwen-hackathon-2026)).
- Brief: build around **model routing** — requests flow through SIE, heavy tasks offload to Alibaba Cloud Model Studio; every model call on Qwen. Tracks: main (Superlinked), visual (Alibaba), social.
- Prizes: cloud/Model Studio coupons ($300/$200/$100) + DJI Mic Mini 2; a Luma listing also mentions $1,000 / $500 Superlinked Cloud credits (figures differ between listings — **partly unverified**; [Luma](https://luma.com/3ssiuf0l)).
- Judges: **not found**.

| Result | Project | What it did | SIE use | Link |
|---|---|---|---|---|
| **Superlinked track winner** | Darwin Router (Kannappan Sirchabesan) | Training-free self-improving router: kNN over labelled exemplars sends each request to a cheap or reasoning model; low-margin cases run both, `qwen-max` judges blind, the verdict becomes a new exemplar. README reports holdout accuracy 75% → 85% and cost per request halved over the day. | `Qwen/Qwen3-Embedding-4B` embeddings and `Qwen/Qwen3.5-4B` cheap lane on SIE | [repo](https://github.com/hashkanna/darwin-router) |
| Alibaba visual track | AliBC (Dimitrios Koutsoumpos) | Autonomous financial news channel | not specified | [repo](https://github.com/dimknaf/AliBC) |
| Social prize | Echo (Mykyta Yakivets, Oriol Morros Vilaseca) | Meeting recorder | SIE writes notes, extracts entities, embeds, reranks | [repo](https://github.com/yakivets/superlinked_hack) |

Other entries singled out for praise in the recap: **Sieve** (logs every candidate's score, reviewer, latency and estimated saving; one premium render instead of 20 — [repo](https://github.com/ShieldTech-Ltd/sieve)) and **TripWire** (scans AI skills/MCP servers, small model reviews, escalates on disagreement, dashboard records the full decision chain — [repo](https://github.com/neomatrix369/tripwire)). The two Sieve authors later bylined a Superlinked blog post ([agent-action-monitor](https://superlinked.com/blog/agent-action-monitor)).

### Infer-a-thon with Superlinked — virtual, 2026-12-05 to 12-20 (upcoming)
- With Geek Room; tracks "Agentic Businesses in India" and "Agentic Engineering"; ₹1.5 lakh+ pool. Rule: must use SIE for "at least one supporting capability, not just a single embeddings call"; cookbook recommended but "Originality matters." No winners yet ([Luma](https://luma.com/f7ef30j2)).

### Other
- Prior San Francisco hackathons sponsored by Superlinked with SIE prize tracks: **not found**. This appears to be their first SF hackathon prize for SIE Serverless.
- Devpost galleries for any Superlinked track: **not found**.

### Patterns in what wins (one event of evidence — treat as a hint)
1. **Split work by cost**: small model by default, big model only where it changes the result. The recap's central praise.
2. **Numbers on screen**: accuracy and cost before/after, saving per request.
3. **Auditable decision records**: Sieve and TripWire were praised specifically for showing *why* an option won.
4. **No training, no cluster**: clever use of off-the-shelf small models within a day.
5. **Explicit boundary** between what stays on small/local models and what goes to hosted frontier models (confidence, privacy, cost).

---

## 10. Fit with "shopping agent with an audit trail"

Time costs assume one person, Python, Serverless key already working.

| # | Idea | SIE pieces | Time | Audit-trail payoff |
|---|---|---|---|---|
| 1 | **Rerank within hard filters.** Code applies budget/size/stock/shipping filters; reranker orders survivors; held items with missing data are listed. Direct port of the cookbook recipe. | `score` Qwen3-Reranker-4B | ~30 min | Log filter results, `item_id`, `score`, `rank`, threshold, `served_by`. |
| 2 | **Evidence-backed spec extraction.** From product pages, return policies, order emails: pull price, return window, warranty, shipping as spans; drop any value not literally in the source. | `extract` gliner2-large-v1 (+ LightOnOCR-3-4B for screenshots/receipts) | 45–60 min | Each fact carries source URL, quoted span, char offsets, model score. Satisfies "reliable context". |
| 3 | **Decision gates before action.** Typed yes/no and pick-one checks: "meets every stated constraint?", "is this a purchase needing approval?", "does the retailer policy allow return?". Low probability or small margin → ask the shopper instead of acting. | `extract` GLiNER2.5-Decide (or `/v1/decisions`) | 45–60 min | Probabilities and thresholds per gate; visible "asked the user because margin 0.08 < 0.15". |
| 4 | **Guarded harness.** Frontier model plans; SIE tools read; `pii-redact` masks address/card data before the planner sees it; `Qwen3Guard` screens scraped page text for injection; Python verifies every quoted price/claim; human approves checkout (test mode). Package tools as an MCP server so the harness is reusable. | 1–3 plus pii-redact, Qwen3Guard-Gen-4B, optional MCP | 2–3 h | Full step ledger: tool, model, latency, cost, evidence, check pass/fail, approval. Targets the **Harness** track. |
| 5 | **Cost-aware router with a scoreboard.** Route each sub-task to the cheapest model that passes its check (Decide → Qwen3.5-4B / Qwen3.8-27B on SIE → frontier). Show per-run cost and latency against an all-frontier baseline; add photo search with SigLIP ("this lamp, but darker wood"). | chat completions + encode SigLIP + all above | +1.5–2 h on top of 4 | The measured comparison is the **Technical build** pitch and echoes the London winner. |

Recommended scope for 5.5 h: build 1 + 2 + 3 early (about 2.5 h including glue), then the ledger UI and approval gate from 4. Add 5's scoreboard only if the core is stable by hour 4.

Cross-sponsor note: the SIE repo ships a `cognee` example where Cognee memory uses SIE models for LLM, embeddings, OCR, rerank and decision ([example](https://github.com/superlinked/sie/tree/main/examples/cognee)) — one integration could count toward both Cognee and Superlinked. There is also a Stripe test-mode checkout example with an SIE risk gate that writes `metadata.sie_risk_band` onto the PaymentIntent ([doc](https://superlinked.com/docs/examples/stripe-link-fraud)) — a ready pattern for "meaningful action, clearly test mode".

Demo framing that matches the judges' stated preferences: a bounded loop, read-only tools until an explicit approval, one honest failure case caught by a check, and a one-line cost/latency figure.

---

## 11. Risks and open questions for the booth

Risks:
- **Cold starts during the demo** (minutes for rerank/embedding, which are never relayed). Mitigate: warm-up script, keep-alive pings, cached last-good results, or pre-record the video early.
- **Generation limit of 4 in flight per key**, plus a shared generation capacity limit that can return 429 even under your own limits ([doc](https://superlinked.com/docs/cloud/rate-limits)).
- **Everyone at the event hitting the same models at once** — unknown capacity behaviour.
- **Extract models have ~512-token windows**; long product pages need windowing.
- **Small-model tool calling is less reliable** than frontier models; keep the planner on a strong model unless routing is the point.
- **Prize criteria are unpublished**; my reading of the two tracks is inference.
- **Model IDs on Serverless**: the catalog page showed some IDs only as URL slugs. Confirm exact IDs via `GET /v1/models` or the console before coding against them.

Questions to ask:
1. What separates the two tracks, and can one project win both? Does "harness" mean our own agent loop, or SIE plugged into an existing harness (Claude Code, OpenCode, etc.)?
2. Must track (a) use Serverless exclusively? Is SIE Local/Cluster acceptable for track (b)?
3. Is a frontier model as planner fine, or do they prefer the whole loop on SIE-served open models?
4. Are there hackathon credits or a code beyond the $20? Is the "$500 more" grant instant?
5. Which models are kept warm today? Expected cold-start time for Qwen3-Reranker-4B, Qwen3-Embedding-4B, GLiNER2.5-Decide, LightOnOCR-3-4B?
6. Any raised rate limits for the event?
7. Does the `sie_mcp` edge work against `api.superlinked.com`, or should we write our own MCP server as in the cookbook?
8. Is `/v1/decisions` or `client.extract(..., output_schema={"type":"choice"})` the preferred way to call Decide?
9. What would make a project cookbook-worthy? (Signals what they will score.)
10. How will they judge — live demo, repo read, or both? Do they want numbers (cost/latency) in the README?

---

## 12. Sources

Official product and docs
- https://superlinked.com/serverless
- https://superlinked.com/docs (and `/docs/index.md`)
- https://superlinked.com/llms.txt · https://superlinked.com/llms-full.txt · https://superlinked.com/llms-small.txt
- https://superlinked.com/docs/quickstart
- https://superlinked.com/docs/reference/api
- https://superlinked.com/docs/cloud/sie-relay
- https://superlinked.com/docs/cloud/rate-limits
- https://superlinked.com/docs/cloud/retries
- https://superlinked.com/docs/generate/tool-calling
- https://superlinked.com/docs/migrate/openai
- https://superlinked.com/docs/migrate/openai-decisions
- https://superlinked.com/docs/extract
- https://superlinked.com/docs/examples · https://superlinked.com/docs/examples/stripe-link-fraud
- https://superlinked.com/models
- https://superlinked.com/about
- https://superlinked.com/e-commerce

Blog
- https://superlinked.com/blog
- https://superlinked.com/blog/launch
- https://superlinked.com/blog/qwen-hackathon-london
- https://superlinked.com/blog/small-open-models-as-agent-tools
- https://superlinked.com/blog/agentic-vs-workflows-small-models
- https://superlinked.com/blog/ecommerce-search-and-catalog-with-small-models
- https://superlinked.com/blog/small-open-models-across-industries
- https://superlinked.com/blog/choosing-where-small-models-run
- https://superlinked.com/blog/carve-off-tasks-not-models
- https://superlinked.com/blog/building-legal-ai-agents-with-small-models
- https://superlinked.com/blog/kubecon-na-2026-inference
- https://superlinked.com/blog/serve-private-document-tools-sie-mcp
- https://superlinked.com/blog/agent-action-monitor

GitHub
- https://github.com/superlinked (org)
- https://github.com/superlinked/cookbook (cloned; README, AGENTS.md, MODELS.md, `.env.example`, `personal/*`, `ecommerce/*`, `retail/*`, `agentic/*`, `legal/redact_documents_with_log`, `security/screen_assistant_prompts`)
- https://github.com/superlinked/sie (README; `examples/` listing and latest release via the GitHub API)
- https://github.com/superlinked/sie/tree/main/examples/cognee
- https://github.com/superlinked/superlinked (archived framework)
- https://github.com/hashkanna/darwin-router
- https://github.com/ShieldTech-Ltd/sieve · https://github.com/neomatrix369/tripwire · https://github.com/yakivets/superlinked_hack · https://github.com/dimknaf/AliBC (linked from the recap; READMEs not individually read except darwin-router)

Event and hackathons
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack
- https://www.hackathonradar.com/database/hackathon/superlinked-x-qwen-hackathon-2026
- https://luma.com/3ssiuf0l (search snippet only)
- https://luma.com/f7ef30j2

Third-party (lower confidence)
- https://nordic9.com/news/superlinked-raised-a-95-million-seed-round-led-by-index-ventures-with-additional-investors-including-theory-ventures-20sales-and-firestreak
- https://www.cbinsights.com/company/superlinked/financials
- https://theorg.com/org/superlinked/org-chart/daniel-svonava
- https://yespress.io/superlinked-inc · https://yespress.io/daniel-svonava
- https://podwise.ai/episodes/8970193
- https://home.mlops.community/public/videos/declarative-mlops-streamlining-model-serving-on-kubernetes
- https://digg.com/ai/eri8ef68
