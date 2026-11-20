# Cognee — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (2026-11-20, SF). Challenge: "build a shopping agent with an audit trail".
> Everything below is sourced from Cognee's live docs, GitHub and public event pages unless marked **unverified** / **not found**.
> Version context: `cognee` 1.6.10 on PyPI, released 2026-11-18 (the day before this research) — [changelog](https://docs.cognee.ai/changelog), [PyPI](https://pypi.org/project/cognee/).

---

## 1. TL;DR

- **What it is:** an Apache-2.0 open-source "memory layer" that turns text/files/URLs into a knowledge graph + vector index and serves it back to agents. Since v1.0 the API is four verbs: `remember`, `recall`, `improve`, `forget`. `add` / `cognify` / `search` / `memify` still work but the docs now file them under "Legacy Operations" ([intro](https://docs.cognee.ai/getting-started/introduction), [Python API](https://docs.cognee.ai/python-api)).
- **What the prize wants:** the track is just "Best use of cognee" with no published rubric. Every prior Cognee-run rubric rewards the same things: memory that is *structured* (datasets + node sets), that *improves from use* (sessions → `improve`, feedback), that can *forget / correct*, and a demo that makes the memory visibly do work ([cognee-hackathons repo](https://github.com/topoteretes/cognee-hackathons)). The organizers linked one cookbook — Personalized Email — which is a template for exactly this: several sources → one dataset, tagged by node set, then two scoped `recall` calls that produce an action grounded in remembered facts ([cookbook](https://docs.cognee.ai/cookbooks/personalized-email)).
- **Fastest qualifying integration (~30–45 min):** `pip install cognee`, set `LLM_API_KEY`, `remember()` three things into one dataset with three node sets (`preferences`, `purchase_history`, `retailer_policies`), then `recall(..., node_name=[...], include_references=True)` before each agent decision and log the returned `Evidence:` block. That is a direct port of the linked cookbook.
- **What makes it a winner rather than a qualifier:** use Cognee as the *audit trail itself*, not only as context. Cognee ships three audit primitives most teams will miss: `include_references=True` (chunk/data ids behind an answer), agent **session traces** (`@cognee.agent_memory(save_session_traces=True)` + `recall(scope="trace")`), and an opt-in **hash-chained provenance ledger** (`PROVENANCE_TRACKING=true`, `verify_chain()`).
- **Judge fit:** Veljko Kovac (Head of Forward Deployed Engineering) did graph-neural-network research at Cambridge and DeepMind and previously founded a startup building a *governance layer for AI agents*; his current talks are titled "Memory Without Deterministic Coordinates Is Just Hearsay" ([AI Dev NYC bio](https://ai-dev.deeplearning.ai/speakers/veljko-kovac), [Tessl DevCon](https://tessl.io/devcon/speakers)). An audit trail where every decision points at a deterministic id (data_id, chunk_id, ledger row) is squarely his thesis.

---

## 2. What they do

### Product

Cognee is "a memory layer for AI agents and tools. It is not an LLM, and it is more than plain RAG" — it sits between your data and whatever model you use, and adds a knowledge graph of entities and relationships on top of chunk embeddings so recall can follow connections ([intro](https://docs.cognee.ai/getting-started/introduction)).

Surfaces:

| Surface | Notes | Source |
|---|---|---|
| Python SDK (`cognee`) | Primary. Python 3.10–3.14. Fully async. | [install](https://docs.cognee.ai/getting-started/installation) |
| Cognee Cloud | Managed tenant at `platform.cognee.ai`; same `/api/v1/*` HTTP API as self-hosted | [cloud HTTP API](https://docs.cognee.ai/cognee-cloud/http-api) |
| REST API | `POST /api/v1/remember`, `/recall`, `/add`, `/cognify`, `/search`, `/datasets/*`, `/provenance/*` | same |
| MCP server | Hosted at `https://mcp.aws.cognee.ai/mcp` (OAuth, no API key) or self-run `cognee-mcp` | [Cloud MCP](https://docs.cognee.ai/cognee-cloud/connections/cloud-mcp) |
| TypeScript SDK (`@cognee/cognee-ts`) | Node bindings over the Rust engine `cognee-rs` (Neon). npm latest is 0.2.0 (2026-07-30) — much younger than the Python SDK | [TS docs](https://docs.cognee.ai/typescript/getting-started), [npm](https://www.npmjs.com/package/@cognee/cognee-ts) |
| CLI | `cognee-cli remember / recall / forget / demo / provenance …` | [quickstart](https://docs.cognee.ai/getting-started/quickstart) |
| Agent plugins | Claude Code, Codex, OpenClaw, Hermes, Cursor, Cline, n8n, Dify, LangGraph, CrewAI, Google ADK, Strands | [integrations index](https://docs.cognee.ai/llms-integrations.md) |

### How it works technically

- **Three stores.** Relational (SQLite default), vector (LanceDB default), graph (Ladybug default; the Kuzu fork). No external servers needed for a local install ([install](https://docs.cognee.ai/getting-started/installation)). Alternatives: Postgres/pgvector, Neo4j, Qdrant, FalkorDB, Redis, Neptune, Turso, etc. ([core docs index](https://docs.cognee.ai/llms-core.md)).
- **`remember`** = ingest + chunk + LLM entity/relation extraction + graph build + an automatic `improve()` pass. With `session_id` it instead writes to a fast session cache ([remember()](https://docs.cognee.ai/python-api/remember)).
- **`recall`** = retrieval. Auto-routes by rule (no LLM call) — default route is `HYBRID_COMPLETION` (chunks + summaries + graph entities/facts → one LLM answer). Can search graph, session, traces, or all ([recall()](https://docs.cognee.ai/python-api/recall)).
- **`improve`** = enrichment: bridges session Q&A / traces / learned guidance into the permanent graph, applies feedback weights, builds a global context index, and reviews conflicting facts ([improve()](https://docs.cognee.ai/python-api/improve)).
- **`forget`** = delete an item, a dataset, or everything ([forget()](https://docs.cognee.ai/python-api/forget)).
- **Datasets** are the named memory containers (isolation + permissions); **node sets** are tags that become real `NodeSet` nodes with `belongs_to_set` edges, used to scope recall ([node sets](https://docs.cognee.ai/core-concepts/further-concepts/node-sets)).
- **Temporal:** dates are extracted by default as `Timestamp` nodes; `SearchType.TEMPORAL` answers before/after/between questions ([temporal guide](https://docs.cognee.ai/guides/temporal-knowledge-graph)).
- **Ontologies:** OWL (RDF/XML) files ground extracted entities to canonical classes ([ontology guide](https://docs.cognee.ai/guides/ontology-support)).
- **Versioning:** every node has `version`, `created_at`, `updated_at`; conflicting facts are kept and labelled `current` / `superseded` / `conflicting`; per-fact extraction evidence is stored by default; a document-level audit ledger is opt-in ([memory versioning](https://docs.cognee.ai/core-concepts/further-concepts/memory-versioning)).

### Positioning and competitors

- Tagline on the event page: "Open-source memory for AI agents." Docs positioning: knowledge-graph-first, self-hostable, runs fully local (even LLM-free via GLiNER / Ollama) ([README](https://github.com/topoteretes/cognee)).
- Published self-reported benchmarks: 0.79 on BEAM 100K, 0.67 on BEAM 10M (exploratory), 91.3% on LoCoMo; results and judge verdicts are in the repo ([accuracy benchmarks](https://docs.cognee.ai/core-concepts/evaluation-and-efficiency/accuracy-benchmarks)). These are vendor-run.
- Competitors, per third-party and vendor comparison guides (treat all as partial): **Mem0** — simplest per-user memory, largest community; **Zep / Graphiti** — temporal knowledge graph, strongest when facts change over time; **Letta** — agent runtime where the agent self-edits tiered memory; **Cognee** — graph-native, local-first ([mcp.directory comparison](https://mcp.directory/blog/mem0-vs-letta-vs-zep-vs-cognee-2026), [Cognee's own comparison](https://www.cognee.ai/blog/guides/open-source-memory-frameworks-llm-agents), [vectorize.io](https://vectorize.io/articles/cognee-alternatives)). No vendor-independent head-to-head benchmark found.
- Research paper: "Optimizing the Interface Between Knowledge Graphs and LLMs for Complex Reasoning" ([arXiv:2505.24478](https://arxiv.org/abs/2505.24478)).

---

## 3. Company facts

| Fact | Detail | Source |
|---|---|---|
| Legal / GitHub org | Topoteretes; repo `topoteretes/cognee`, created 2023-08-16 | [GitHub API](https://api.github.com/repos/topoteretes/cognee) |
| Founder / CEO | Vasilije Markovic | [about](https://www.cognee.ai/about-us) |
| Offices | Berlin (Kreuzberg) and San Francisco (1288 Howard St) | [about](https://www.cognee.ai/about-us) |
| Seed round | $7.5M, announced 2026-02-19, led by Pebblebed, with 42CAP and Vermilion Ventures; angels from Google DeepMind, n8n, Snowplow | [Cognee blog](https://www.cognee.ai/blog/cognee-news/cognee-raises-seven-million-five-hundred-thousand-dollars-seed) |
| Earlier funding | An earlier ~$1.5M is mentioned in search results for a Cognee blog post — **unverified**, not read directly | [blog](https://www.cognee.ai/blog/cognee-news/funding-and-web-site-launch) |
| Traction (as of Feb 2026 post) | Pipelines grew from ~2,000 runs to 1M+ in 2025; live in 70+ companies | seed post above |
| GitHub | 33.4k stars, 3.5k forks (2026-11-19); Apache-2.0 | [GitHub API](https://api.github.com/repos/topoteretes/cognee) |
| Team listed on site | ~16 people incl. Igor Ilic (Head of SDK), Nikola Zivkovic (VP Engineering), Vincent Pistor (VP Commercial), Lazar Obradovic (Data Science Lead), Veljko Kovac (Head of FDE) | [about](https://www.cognee.ai/about-us) |
| Accelerator | UC Berkeley Xcelerator 2026 Agentic AI cohort (title only, not read) | [blog](https://www.cognee.ai/blog/cognee-news/cognee-berkeley-xcelerator-2026-cohort) |
| Release cadence | v1.0.0 introduced `remember/recall/improve/forget`; now at v1.6.10 (2026-11-18), with very frequent point releases | [changelog](https://docs.cognee.ai/changelog) |

Note: aggregators disagree on currency ($ vs €7.5M) and on co-investor names (one lists "Combination VC"). The table uses Cognee's own announcement.

---

## 4. Customers and use cases

Named on Cognee's own customers page ([customers](https://www.cognee.ai/customers)):

| Customer | What they did | Quoted metric |
|---|---|---|
| Bayer | Moved from a chatbot to an agentic research system on Cognee memory (scientific research workflows) | none on the page |
| University of Wyoming | Agentic teaching assistant / evidence graph from policy documents | "first memory system live in 30 days" |
| Knowunity | Knowledge graph connecting learners | POC on 40,000 students in 2 days |
| Dynamo (dynamo.fyi) | Enriched data for personalised support; one of the first production deployments ([case study, Nov 2024](https://cognee.ai/blog/case-studies/cognee-case-study-with-dynamo)) | built and deployed within a month |
| SlideSpeak | Shared-context memory for slide creation | none |
| Luccid | Queryable knowledge base of building codes and regulations | none |
| DeepMetis | On-prem AI memory | "significantly increased" accuracy (no number) |
| Unnamed Tier-1 US bank | Credit-card insights | none |

Also named in the seed announcement: Dilbloom and dltHub "integrated into their stacks" ([seed post](https://www.cognee.ai/blog/cognee-news/cognee-raises-seven-million-five-hundred-thousand-dollars-seed)).

Verticals Cognee documents with worked examples: agentic procurement decisions, live-context support agent, company brain, HR resume screening, developer knowledge base, edge AI ([examples index](https://docs.cognee.ai/llms-core.md)). **No retail / e-commerce / shopping customer or example was found** — the closest is the procurement example (section 10).

---

## 5. Why they are at this hackathon

Inferred from recent launches and DevRel pattern, not stated by Cognee:

1. **Drive Cognee Cloud signups and credit usage.** The event's "Cognee Cloud" resource link goes straight to `platform.cognee.ai/sign-up`, and two of three prizes are paid in Cognee credits ([event page](https://tokensand.com/personalagenthack)). Their own Berlin hackathon said outright that using Cloud "boosts your reward" with a dedicated "Best use of Cognee Cloud" bonus ([Cloud hackathon README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-cloud-hackathon-2026-06-19)). Billing docs have a first-class "hackathon code" redemption flow ([billing](https://docs.cognee.ai/cognee-cloud/functionality/account-and-billing)).
2. **Get developers onto the v1 verbs.** The linked cookbook is built entirely on `remember` / `recall` with node sets — not `add` / `cognify` / `search`. Using the legacy names in the demo signals you learned Cognee from a 2025 blog post.
3. **Collect product feedback.** The hackathons repo ships a `cognee-hackathon-feedback` agent skill that records "errors, slow steps, confusion" for participants to send the team ([repo README](https://github.com/topoteretes/cognee-hackathons)). Telling the booth what broke is welcome, not rude.
4. **Push the "memory that improves" narrative.** Three of their 2026 hackathons made session → graph distillation "the core piece" judges look at ([Redis](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-redis-hackathon-2026-05-16), [Cloud](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-cloud-hackathon-2026-06-19), [Daytona/Moss](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-daytona-moss-hackathon-2026-04-25)).
5. **Personal-memory use cases.** The current cookbook set is all personal/company-brain agents (personalized email, follow-up agent, Google Workspace memory, self-hosted companion) ([core docs index](https://docs.cognee.ai/llms-core.md)) — a personal shopping agent extends that line.
6. **SF presence.** They have an SF office and have run several SF events in 2026 (Feb AI-Memory Hackathon, Mem-Off at Pebblebed, SF Tech Week Company Brain on Oct 7).

---

## 6. Prize track

**Exact wording** ([event page](https://tokensand.com/personalagenthack)):

> **Best use of cognee**
> - 1st place: $500 USD cash + $500 in Cognee Credits
> - 2nd place: $250 USD cash + $500 in Cognee Credits
> - 3rd place: $500 in Cognee Credits

Resources linked by organizers: Docs (`docs.cognee.ai`), Cookbooks (`docs.cognee.ai/cookbooks/personalized-email`), Cognee Cloud (`platform.cognee.ai/sign-up`), Cognee SDK (`github.com/topoteretes/cognee`).

No track-specific judging criteria are published — **not found** on the event page or the Luma page ([Luma](https://luma.com/personalagenthack)).

### People

**Veljko Kovac — Head of FDE (Forward Deployed Engineering), Cognee. Judge and speaker at this event** ([Luma](https://luma.com/personalagenthack)).

- Leads technical rollout of agent memory for enterprise clients, moving memory layers from prototype to production ([AI Dev NYC speaker page](https://ai-dev.deeplearning.ai/speakers/veljko-kovac)).
- Previously founded a venture-backed startup building a **governance layer for AI agents** (raised in London and SF) — company name **not found** (same source).
- Research background: graph neural networks at the University of Cambridge with Prof. Pietro Liò and at Google DeepMind with Dr. Petar Veličković (same source).
- Upcoming/recent talks: "Memory Without Deterministic Coordinates Is Just Hearsay" ([Tessl AI Native DevCon](https://tessl.io/devcon/speakers)); "Memory for agents, teams, and people" workshop, AI Dev NYC, Dec 1 2026 ([DeepLearning.AI](https://ai-dev.deeplearning.ai/)). A second DevCon title, "Same Repo, Different Failures: Deterministic Memory for Coding Agents", appeared in search results only — **unverified**.
- Writing: "Memory as a Harness: Turning Execution Into Learning" (2026-03-24) — "storing interactions is not the same as learning"; memory is the harness layer that records execution, attaches outcomes and feeds them back ([post](https://www.cognee.ai/memory-as-a-harness-turning-execution-into-learning)). "Separate Memories for Organization, Agent and User" (2026-05-06) — user / agent / org memory scopes with permissions and human-in-the-loop ([post](https://www.cognee.ai/separate-memories-for-organization-agent-and-user)).
- What this implies for a pitch: expect questions on *how you know* a remembered fact is right, where it came from, what happens when it changes, and whether memory is scoped per user. He is unlikely to be impressed by "we stored the chat history".

**Vasilije Markovic — CEO and Founder.** Not listed as attending this event. Judged the Feb 2026 SF AI-Memory Hackathon ([Luma](https://luma.com/50si7fw4)); AI Engineer speaker ([profile](https://www.ai.engineer/speakers/vasilije-markovic), not read); co-author of the arXiv paper above. Further background **not researched in depth**.

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **Evidence-carrying recall as the audit trail.** `recall(..., include_references=True)` appends a deterministic `Evidence:` block and, for `RAG_COMPLETION` / `GRAPH_COMPLETION`, fills `metadata.evidence` with chunk / node / edge references (`data_id`, `chunk_id`, `document_name`). `visualize_graph(..., recall_result=result)` renders the subgraph behind one specific answer ([recall() — source provenance](https://docs.cognee.ai/python-api/recall), [graph visualization](https://docs.cognee.ai/guides/graph-visualization)). This maps one-to-one onto "show the information and steps behind what it did" and onto Kovac's "deterministic coordinates" theme.
2. **Node sets for separable memory categories.** Preferences vs purchase history vs retailer policy vs product catalog, each recalled in isolation with `node_name=[...]`. This is the core trick of both the linked cookbook and the procurement example ([cookbook](https://docs.cognee.ai/cookbooks/personalized-email), [procurement](https://docs.cognee.ai/examples/agentic-procurement)).
3. **Sessions + `improve()` (memory that gets better).** `recall(session_id=...)`, `cognee.session.add_feedback(...)`, then `improve(session_ids=[...])`; or `AUTO_FEEDBACK` so stated preferences ("never over $200", "no fast fashion") become session guidance without an explicit API call ([feedback guide](https://docs.cognee.ai/guides/feedback-system), [live-session example](https://docs.cognee.ai/examples/live-session-feedback)). This is "the core piece" in three earlier Cognee rubrics.
4. **Agent session traces.** `@cognee.agent_memory(save_session_traces=True)` on tool functions records each call (params, return value, error, status); `recall(scope="trace")` reads them back ([traces guide](https://docs.cognee.ai/guides/agent-session-traces)). An audit trail of *actions*, stored in the same memory as the facts.
5. **Provenance ledger.** `PROVENANCE_TRACKING=true` → append-only, SHA-256 hash-chained ledger row per stored document; `verify_chain()`, `cognee-cli provenance verify / history / export`; tombstones on delete ([cognify() — provenance ledger](https://docs.cognee.ai/python-api/cognify)). A tamper-evident "what did the agent know and when" record. Rare in hackathon demos.
6. **Conflict review / versioning.** Price changed, policy updated: `improve()` keeps both facts, labels them `current` / `superseded` / `conflicting`; `cognee.list_conflicts()` / `resolve_conflict()` let a human settle it (since 1.6.6) ([conflicts](https://docs.cognee.ai/python-api/conflicts), [versioning](https://docs.cognee.ai/core-concepts/further-concepts/memory-versioning)).
7. **`forget()` as a product feature.** The Open Source grand winner of their biggest hackathon was built around provable forgetting (section 9). For shopping: "forget that I was shopping for a gift for X" / revoke a retailer's data.
8. **Temporal recall.** `SearchType.TEMPORAL` for "what did I buy before June?" / "price as of last week". Slow (~40 s median in their own benchmark) — use sparingly ([search types — speed](https://docs.cognee.ai/python-api/search-type)).
9. **Cognee Cloud.** `cognee.serve(url=..., api_key=...)`; previously carried a rubric bonus. Also gives you a hosted graph UI and Activity log to show judges ([cloud SDK](https://docs.cognee.ai/cognee-cloud/connections/cloud-sdk)).
10. **Ontology grounding.** A small OWL file of product categories / attributes so "trainers" and "sneakers" land on one class ([ontology guide](https://docs.cognee.ai/guides/ontology-support)). Impressive but costly to author in 5.5 h.
11. **MCP server.** Fine for wiring Cognee into Claude/Cursor while building; as the only integration it reads as shallow.

---

## 8. Quickstart

### 8a. Local (open source)

```bash
python -m venv .venv && source .venv/bin/activate
pip install cognee            # or: uv pip install cognee
echo 'LLM_API_KEY="your_openai_api_key"' >> .env
```

- Python 3.10–3.14. Default stores (SQLite + LanceDB + Ladybug) need no servers ([install](https://docs.cognee.ai/getting-started/installation)).
- `LLM_API_KEY` alone is enough for OpenAI, which is the default for **both** LLM and embeddings (`openai/text-embedding-3-large`, 3072 dims). The docs currently name the default LLM as `openai/gpt-5.6-luna` (same source).
- **Gotcha:** if you switch the LLM provider but not embeddings, embeddings still default to OpenAI and need an OpenAI key. Set both `LLM_*` and `EMBEDDING_*`:

```bash
# Example from the docs (Gemini)
LLM_PROVIDER="gemini"
LLM_MODEL="gemini/gemini-flash-latest"
LLM_API_KEY="your_gemini_api_key"
EMBEDDING_PROVIDER="gemini"
EMBEDDING_MODEL="gemini/gemini-embedding-001"
EMBEDDING_API_KEY="your_gemini_api_key"
```

- Anthropic needs `pip install "cognee[anthropic]"` (and still a separate embedding provider) ([install](https://docs.cognee.ai/getting-started/installation)).
- No key at all: `cognee-cli demo` proves the install with a prebuilt graph and keyword search ([quickstart](https://docs.cognee.ai/getting-started/quickstart)).

Minimal script, verbatim from the docs ([quickstart](https://docs.cognee.ai/getting-started/quickstart)):

```python
import cognee
import asyncio

async def main():
    # Create a clean slate for cognee -- reset data and system state
    await cognee.forget(everything=True)

    # Store content in memory (ingests, builds knowledge graph, enriches)
    text = "Cognee turns documents into AI memory."
    await cognee.remember(text)

    # Retrieve from memory
    results = await cognee.recall(
        query_text="What does Cognee do?"
    )

    # Print
    for result in results:
        print(result.text)

if __name__ == '__main__':
    asyncio.run(main())
```

### 8b. Cognee Cloud

1. Sign up at `https://platform.cognee.ai/` (Google, GitHub or email). Workspace provisioning "can take a couple of minutes"; if sign-ups are capped you land on a waitlist ([cloud quickstart](https://docs.cognee.ai/cognee-cloud/quickstart)). **Do this the night before.**
2. New workspaces get **$10 free credit, no card**. Pricing is $0.10 per 1M tokens; ingestion costs roughly 2.5× the input tokens. Hackathon/voucher codes are redeemed under Billing → Add credits → "Have a code?" ([billing](https://docs.cognee.ai/cognee-cloud/functionality/account-and-billing)).
3. Copy the **API base URL** (`https://tenant-<id>.aws.cognee.ai` or `.gcp.`) and create a key on the API Keys page ([HTTP API](https://docs.cognee.ai/cognee-cloud/http-api)).
4. On Cloud you do **not** supply your own LLM key for the tenant's processing — usage is billed from credit (inferred from the billing model; confirm at the booth).

Python, verbatim ([cloud SDK](https://docs.cognee.ai/cognee-cloud/connections/cloud-sdk)):

```python
import asyncio
import cognee

async def main():
    # Connect to Cognee Cloud
    await cognee.serve(
        url="https://your-tenant.aws.cognee.ai",
        api_key="your-api-key"
    )

    # Store content in memory — ingests, builds knowledge graph, enriches
    await cognee.remember(
        "Cognee Cloud automates knowledge graph creation in the cloud.",
        dataset_name="default_dataset",
    )

    # Retrieve from memory
    results = await cognee.recall(
        query_text="What does Cognee Cloud automate?",
    )
    for result in results:
        print(result)

    # Disconnect when done
    await cognee.disconnect()

asyncio.run(main())
```

HTTP (works from any language, including a TypeScript/Next.js app) ([cloud quickstart](https://docs.cognee.ai/cognee-cloud/quickstart), [data ingestion](https://docs.cognee.ai/cognee-cloud/functionality/data-ingestion)):

```bash
# health + auth check
curl https://your-tenant.aws.cognee.ai/health
curl https://your-tenant.aws.cognee.ai/api/v1/datasets/ -H "X-Api-Key: your-api-key"

# remember (multipart form; text goes in raw_data)
curl -X POST "$COGNEE_BASE_URL/api/v1/remember" \
  -H "X-Api-Key: $COGNEE_API_KEY" \
  -F "raw_data=Shopper prefers vegan leather and never spends over 200 USD on shoes." \
  -F "datasetName=shopper_memory" \
  -F "node_set=preferences" \
  -F "run_in_background=false"

# recall (JSON)
curl -X POST "$COGNEE_BASE_URL/api/v1/recall" \
  -H "X-Api-Key: $COGNEE_API_KEY" -H "Content-Type: application/json" \
  -d '{"query": "What do you know?", "datasets": ["my_dataset"]}'
```

The `remember` example above is adapted from the documented parameter table (field names `raw_data`, `datasetName`, `node_set`, `run_in_background` are documented; the exact multipart encoding of the `node_set` array was not tested — verify once before relying on it).

### 8c. Current signatures (v1.6.x)

```python
# https://docs.cognee.ai/python-api/remember
async def remember(
    data,                               # str | list[str] | UploadFile | DataItem | MemoryEntry | MemorySource | dict ...
    dataset_name: str = "main_dataset",
    *,
    dataset_id: Optional[UUID] = None,
    session_id: Optional[str] = None,
    chunk_size: Optional[int] = None,
    chunker: Optional[Any] = None,
    custom_prompt: Optional[str] = None,
    run_in_background: bool = False,
    self_improvement: bool = True,
    session_ids: Optional[List[str]] = None,
    dry_run: Union[bool, Literal["presort"]] = False,
    raise_on_error: bool = True,
    **kwargs,                           # node_set=[...], config={...} are passed here in the docs' examples
) -> Union[RememberResult, DryRunEstimate, PresortReport, dict]

# https://docs.cognee.ai/python-api/recall
async def recall(
    query_text: str,
    query_type: SearchType | None = None,
    *,
    datasets: list[str] | None = None,
    dataset_ids: list[UUID] | None = None,
    top_k: int = 15,
    auto_route: bool = True,
    scope: str | list[str] | None = None,   # "graph" | "session" | "trace" | "session_context" | "session_first" | "all" | "auto"
    # keyword-only extras: system_prompt, node_name, node_type, node_name_filter_operator,
    # only_context, session_id, context_profile, feedback_influence, verbose,
    # response_model, include_references, user, llm_config, embedding_config, ...
) -> list[RecallResponse]

# https://docs.cognee.ai/python-api/improve
async def improve(
    dataset: Union[str, UUID] = "main_dataset",
    *,
    run_in_background: bool = False,
    node_name: Optional[List[str]] = None,
    session_ids: Optional[List[str]] = None,
    build_global_context_index: Optional[bool] = None,
    build_truth_subspace: bool = False,
    review_conflicts: Optional[bool] = None,
    **kwargs,
) -> ImproveResult

# https://docs.cognee.ai/python-api/forget
async def forget(
    *,
    data_id: Optional[UUID] = None,
    dataset: Optional[str] = None,
    dataset_id: Optional[UUID] = None,
    everything: bool = False,
    memory_only: bool = False,
    clean_history: bool = False,
    user=None,
) -> dict
```

`SearchType` values ([reference](https://docs.cognee.ai/python-api/search-type)): `SUMMARIES`, `CHUNKS`, `CHUNKS_LEXICAL`, `RAG_COMPLETION`, `HYBRID_COMPLETION` (default), `HYBRID_COMPLETION_DECOMPOSITION`, `GRAPH_COMPLETION`, `GRAPH_COMPLETION_DECOMPOSITION`, `GRAPH_SUMMARY_COMPLETION`, `GRAPH_COMPLETION_COT`, `GRAPH_COMPLETION_CONTEXT_EXTENSION`, `TEMPORAL`, `CYPHER`, `NATURAL_LANGUAGE`, `CODING_RULES`, `CODE`, `GRAPH_REPORT`, `BROAD`, `SKILLS`, `AGENTIC_COMPLETION`, `FEELING_LUCKY`. `TRIPLET_COMPLETION` is deprecated and due for removal. Import: `from cognee import SearchType` (the cookbook uses `from cognee.modules.search.types import SearchType`).

### 8d. The cookbook pattern (the calls the organizers pointed at)

From `examples/cookbooks/personalized_email/scripts/` on the `dev` branch ([ingest_granola.py](https://github.com/topoteretes/cognee/blob/dev/examples/cookbooks/personalized_email/scripts/ingest_granola.py), [draft.py](https://github.com/topoteretes/cognee/blob/dev/examples/cookbooks/personalized_email/scripts/draft.py)):

```python
# ingest: one dataset, one node set per source
await cognee.remember(
    notes, dataset_name=DATASET, node_set=["meetings"], self_improvement=False
)

# act: (1) raw chunks from one node set, (2) hybrid completion with a strict system prompt
own_emails = await cognee.recall(
    f"Emails written by {me}",
    query_type=SearchType.CHUNKS,
    datasets=[DATASET],
    node_name=["sent_mail"],
    top_k=3,
)
reply = await cognee.recall(
    f"Write {me}'s reply to this email:\n{email['text']}\n\nExamples of {me}'s own emails:\n"
    + "\n---\n".join(str(chunk.text) for chunk in own_emails),
    query_type=SearchType.HYBRID_COMPLETION,
    datasets=[DATASET],
    system_prompt=DRAFT_PROMPT.format(me=me),
)
print(reply[0].text)
```

Run the sample end to end (LLM key only, no Gmail/Granola) ([cookbook](https://docs.cognee.ai/cookbooks/personalized-email)):

```bash
git clone https://github.com/topoteretes/cognee.git && cd cognee
uv sync
echo 'LLM_API_KEY="your-key"' >> .env
uv run python examples/cookbooks/personalized_email/personalized_email.py --sample
```

Design points worth copying: the prompt forbids inventing "a date, price or promise"; the action is "printed, never sent" (a clear real/simulated boundary — matches the organizers' "show whether the action is real, simulated or test mode"); `--clear` calls `cognee.forget(dataset=...)` so reruns start clean.

### 8e. Audit-trail snippets

```python
# Evidence behind an answer — https://docs.cognee.ai/python-api/recall
res = await cognee.recall(
    "Which running shoe fits the shopper's budget and return-policy needs?",
    query_type=SearchType.GRAPH_COMPLETION,   # RAG_/GRAPH_COMPLETION populate metadata.evidence; HYBRID only adds the text block
    datasets=["shopper_memory"],
    node_name=["preferences"],
    include_references=True,
)
print(res[0].text)                # answer + "Evidence:" bullets
print(res[0].metadata)            # {"evidence": [...]} with data_id / chunk_id

# Subgraph behind that answer — https://docs.cognee.ai/guides/graph-visualization
from cognee import visualize_graph
await visualize_graph("decision_42.html", dataset="shopper_memory", recall_result=res)
```

```python
# Action traces — https://docs.cognee.ai/guides/agent-session-traces
@cognee.agent_memory(
    session_id=SESSION_ID,
    with_memory=False,
    with_session_memory=False,
    save_session_traces=True,
    session_trace_summary=False,
)
async def add_to_cart(sku: str) -> str: ...

traces = await cognee.recall(query_text="cart", session_id=SESSION_ID, scope="trace")
for t in traces:
    print(f"[{t.status}] {t.origin_function}({t.method_params}) -> {t.method_return_value or t.error_message}")
```

```python
# Tamper-evident ledger — https://docs.cognee.ai/python-api/cognify (Provenance ledger)
# .env BEFORE first run:  PROVENANCE_TRACKING=true
from cognee.modules.provenance import get_provenance_manager
manager = get_provenance_manager()
report = await manager.verify_chain()
print(report["valid"], report["total_entries"], report["broken_links"])
```

```python
# Feedback loop — https://docs.cognee.ai/guides/feedback-system
entries = await cognee.session.get_session(session_id="my_session", last_n=5)
await cognee.session.add_feedback(session_id="my_session", qa_id=entries[-1].qa_id,
                                  feedback_text="Wrong size", feedback_score=1)
await cognee.improve(dataset="main_dataset", session_ids=["my_session"])
```

### 8f. Latency, cost and gotchas

- **Ingestion is slow.** The docs' own procurement example (four short documents) says "Ingestion takes a few minutes before the first question prints" ([procurement](https://docs.cognee.ai/examples/agentic-procurement)). Pre-ingest catalog/policy data before the demo; never ingest on stage. Use `self_improvement=False` during seeding (the cookbook does) and `run_in_background=True` for writes made mid-conversation.
- **Recall latency (Cognee's own small-corpus benchmark, gpt-5-mini):** `CHUNKS` ~0.3 s, `RAG_/GRAPH_/HYBRID_COMPLETION` ~3.8–3.9 s, `TEMPORAL` ~40 s, `NATURAL_LANGUAGE` ~50 s, `GRAPH_COMPLETION_COT` ~200 s ([search types](https://docs.cognee.ai/python-api/search-type)). Stay on the single-call types for live demos.
- `recall()` before any `remember()` raises `RecallPreconditionError` ([recall()](https://docs.cognee.ai/python-api/recall)).
- `recall()` has no `**kwargs`: `dataset_name=` raises `TypeError`; use `datasets=[...]`. `remember()` takes `dataset_name=`; `improve()` takes `dataset=`. Easy to mix up.
- Results are Pydantic objects locally (`result.text`), but **plain dicts in remote/Cloud mode** (`result["text"]`) ([recall() — remote mode](https://docs.cognee.ai/python-api/recall)).
- In remote mode several params are silently dropped: `feedback_influence`, `node_name_filter_operator`, `auto_route`, `llm_config`, `retriever_specific_config`, etc. `remember(session_ids=...)` raises over `serve()` (same source).
- Over `serve()`, a path string passed to `remember()` is uploaded as literal text — pass an open file object to upload a file ([remember()](https://docs.cognee.ai/python-api/remember)).
- `node_set` must be a list (`["preferences"]`), never a bare string → `InvalidNodeSetError`. Node-set scoping is by node, not by source document, so it is a filter, **not a security boundary**; use separate datasets for per-user isolation ([node sets](https://docs.cognee.ai/core-concepts/further-concepts/node-sets)).
- Re-adding identical content is skipped by incremental loading, so reruns will not duplicate — and will not update tags either (same source).
- `feedback_influence` defaults to 0.0: feedback changes nothing until you set `DEFAULT_FEEDBACK_INFLUENCE` or pass it per call ([feedback](https://docs.cognee.ai/guides/feedback-system)).
- `PROVENANCE_TRACKING` is read once per process — set it before the first call. The ledger records documents, not `improve()` stages ([cognify()](https://docs.cognee.ai/python-api/cognify)).
- `cognee.forget(everything=True)` in sample scripts wipes all local Cognee data. Do not paste it into the demo path.
- Cloud errors: `402` = credit exhausted, `401` = wrong key (Bearer tokens do not work on Cloud; use `X-Api-Key`), `/api` without `/v1` = 404, `429` = rate limit, retry with backoff ([HTTP API](https://docs.cognee.ai/cognee-cloud/http-api)).
- Cloud `POST /remember` defaults to `run_in_background=true` — it returns a `pipeline_run_id` immediately and the data is not yet recallable ([data ingestion](https://docs.cognee.ai/cognee-cloud/functionality/data-ingestion)).
- The TypeScript SDK is a local Rust engine binding, not a thin Cloud client; its cloud mode with no args uses an Auth0 device-code flow that needs a TTY ([TS cloud](https://docs.cognee.ai/typescript/cloud)). For a JS/TS app, calling the Cloud HTTP API with `fetch` is the lower-risk path.
- Release churn: 1.6.10 shipped the day before the event. Pin the version you tested.

---

## 9. Past hackathons Cognee sponsored or ran

| Date | Event | Location | Prizes / tracks | Winners |
|---|---|---|---|---|
| 2026-02-07 | AI-Memory Hackathon #1 ("Build on a Prebuilt AI-Memory Knowledge Graph"), with Qdrant, Distil Labs, DigitalOcean, Pebblebed, Bond AI | San Francisco | $1,200 / $900 / $500 cash; Best use of Qdrant $1,500; Best use of Distil Labs $1,000 credits; Best use of DigitalOcean. Judges incl. Vasilije Markovic and Daniel Scott (Cognee / Brown) ([Luma](https://luma.com/50si7fw4)) | **not found** |
| 2026-04-25 | Cognee × Daytona × Moss: PR Rescue Arena | not stated | Tracks incl. Best Self-Improvement Loop, Best Agent Team ([README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-daytona-moss-hackathon-2026-04-25)) | **not found** |
| 2026-05-16 | AI-Memory Hackathon: Building your own Agent LLM Wiki (with Redis) | SF per search result — **unverified** | $800 / $500 / $200 cash ([README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-redis-hackathon-2026-05-16)) | **not found** |
| 2026-06-16 | Cognee Company Brain: Slack + Granola Knowledge Graph | not stated | not stated | **not found** |
| 2026-06-19 | Cognee Cloud Hackathon: Build Your Company Brain | Berlin | 600 / 400 / 200 EUR; "Best use of Cognee Cloud" bonus ([README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-cloud-hackathon-2026-06-19)) | **not found** |
| 2026-06-26 | Cognee GTM Brain (with modelguide) | Warsaw | not stated | **not found** |
| 2026-06-29 → 07-05 | **The Hangover Part AI: Where's My Context?** (WeMakeDevs × Cognee), online; 8,500 participants, 1,200+ projects, $10,000 in prizes | Online | Best Use of Cognee Open Source; Best Use of Cognee Cloud; Best Blog; top-20 PRs ([event](https://archive.wemakedevs.org/hackathons/cognee)) | **Published — see below** |
| 2026-10-07 | Build a Company Brain (SF Tech Week), with Scalekit and Respan | San Francisco | announced at kickoff; 100-point rubric ([README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-companybrain-scalekit-respan-hackathon-2026-10-07)) | **not found** |
| date not found | Mem-Off! The Open Source AI Memory Showdown (claude-mem × cognee × HelixDB) | Pebblebed, SF | none listed ([Luma](https://luma.com/cognee-cy86)) | **not found** |

Source for the list: [topoteretes/cognee-hackathons README](https://github.com/topoteretes/cognee-hackathons/blob/main/README.md). Winner announcements for the in-person events were searched for and not found.

### Verified winners — The Hangover Part AI ([winners page](https://archive.wemakedevs.org/hackathons/cognee/projects))

- **Lethe — Vinayak Sonthalia (solo). Track 1 Grand Winner, Best Use of Cognee Open Source.** Self-hosted on-call incident-triage assistant that "makes forgetting a product feature": hard-deletes decommissioned systems and proves the deletion with a receipt and a re-query. Runbooks → graph (Kuzu) + vectors (LanceDB); forgetting removes raw files, nodes, edges and embeddings; also uses feedback weights for a reversible "demote". The author benchmarked Cognee's four advertised capabilities against plain RAG at temperature 0. [GitHub](https://github.com/vinayaksonthalia/lethe) · [Demo](https://youtu.be/3840gxTZWxY)
- **Classroom Memory — Rajdeep Singh. Track 2 Grand Winner, Best Use of Cognee Cloud.** Per-student private memory of mastery plus a prerequisite graph that tells teachers what to teach next. Real Cognee Cloud calls for per-student memory; SQLite holds exact mastery state; a verification script checks the live Cloud lifecycle. [GitHub](https://github.com/RajdeepKushwaha5/ClassroomMemory) · [Demo](https://www.youtube.com/watch?v=2AUA_g0S3Ks)
- **RealtyRecall — Mahimai Raja J (solo). Best Blog.** Voice receptionist for solo real-estate agents that recognises returning callers by phone number and surfaces newer listings near what they liked; per-agency isolation with NodeSets; a console shows the memory graph growing during calls. [GitHub](https://github.com/mahimairaja/realtyRecall/) · [Blog](https://mahimairaja.medium.com/your-voice-agent-forgets-you-the-moment-you-hang-up-i-gave-it-a-memory-90ef8fdc8327)

(The prize for Track 2 is listed as an iPhone 17 on the event page and an iPad on the winners page — minor inconsistency in the source.)

Highlighted builds (18 listed) closest to an audit-trail idea ([highlights](https://archive.wemakedevs.org/hackathons/cognee/projects/top)):

- **MemoryScope** — logs which memories each answer cited (via `chunk_id` / `data_id`) and flags answers that depended on later-corrected facts. [GitHub](https://github.com/harshithasompura/memory-scope)
- **Rewind** — debugger that visualises the Cognee graph and time-travels through what the agent knew. [GitHub](https://github.com/DivvSaxena/rewind)
- **Aeg** — gateway that screens memory writes and quarantines poisoned data. [GitHub](https://github.com/OoJae/aeg)
- **SessionZero** — trust-scored memory that flags stale or contradicted memories (Cloud). [GitHub](https://github.com/Aashritha-2005/SessionZERO-agentMemory)
- **GroundTruth / FALSIFY** — belief revision: retract a source and cascade the change.

### Patterns in what wins

1. **A sharp, non-generic thesis about memory.** "Stale memory is the real failure" (Lethe), "a classroom is a memory problem" (Classroom Memory). Not "chatbot with memory".
2. **The whole lifecycle, not only write + read.** Most highlighted builds list `remember, recall, improve, forget` together; forgetting/correction shows up in the majority.
3. **Proof, not claims.** Lethe shipped a benchmark vs plain RAG and deletion receipts; Classroom Memory shipped a script that verifies live Cloud calls. Cognee's own rubrics say "Do not just say 'the agent learned'. Show the before score, feedback, skill diff, and after score" ([Daytona/Moss README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-daytona-moss-hackathon-2026-04-25)).
4. **Memory design is explicit.** SF Tech Week rubric: 15 points for "what goes in the graph, how it's tagged, how it improves"; 20 for per-user access shaping the answer; 20 for evaluation with before/after ([README](https://github.com/topoteretes/cognee-hackathons/tree/main/cognee-companybrain-scalekit-respan-hackathon-2026-10-07)).
5. **Cognee holds the relationships; something else holds exact state.** Classroom Memory used SQLite for exact mastery values and Cognee for graph reasoning. For shopping: keep prices/SKUs/cart in a normal store, keep preferences/history/policy relationships in Cognee.
6. **The graph is visible in the UI.** RealtyRecall showed the graph growing live.
7. **Solo and small teams win** — tight scope beats breadth.

Caveat: these patterns come from one week-long online event with published winners plus rubrics from other events. Winners of the short in-person events (the closest analogue to a 5.5 h build) were not found.

---

## 10. Fit with "shopping agent with an audit trail"

The challenge's three requirements map cleanly: (1) *reliable context* → scoped `recall` over node sets; (2) *meaningful action* → your cart/checkout tool, wrapped in a trace decorator; (3) *show the information and steps* → evidence references + traces + ledger.

Closest official template: **Agentic Procurement Decisions** — vendor conversations, purchase history and procurement policy in three node sets; nine research questions each scoped to one category with `GRAPH_COMPLETION`; evidence compiled; one final LLM call recommends a vendor "justified from the evidence" ([doc](https://docs.cognee.ai/examples/agentic-procurement), [script](https://github.com/topoteretes/cognee/blob/main/examples/demos/agentic/agentic_reasoning_procurement_example.py)). That is a B2B shopping agent already. Start from it.

| # | Idea | What you build | Cognee features | Time in a 5.5 h build |
|---|---|---|---|---|
| A (shallow) | **Shopper preference memory** | Seed one dataset with three node sets: `preferences` (sizes, budget, brands to avoid), `purchase_history`, `retailer_policies`. Before each recommendation, one `recall(..., node_name=[...])`; show the answer. | `remember`, `recall`, node sets, datasets | ~30–45 min. Port of the linked cookbook. Qualifies; will not stand out. |
| B | **Cited decisions** | Every agent decision stores `{decision, recall text, Evidence block, metadata.evidence ids}` in your own decision log and renders it in a side panel; "why?" button opens `visualize_graph(recall_result=...)` HTML for that decision. Use `GRAPH_COMPLETION` or `RAG_COMPLETION` so `metadata.evidence` is populated. | `include_references=True`, `metadata.evidence`, `visualize_graph` | +45–60 min on top of A. Best value per minute. |
| C | **Actions in the same memory** | Wrap `search_products`, `add_to_cart`, `checkout_simulated` with `@cognee.agent_memory(save_session_traces=True)`. Audit view = `recall(scope="trace", session_id=...)` — params, return value, errors, status per call. Then `improve(session_ids=[...])` so today's purchase becomes tomorrow's `purchase_history`. | agent session traces, sessions, `improve` | +45–60 min. Local SDK only (decorator); test `scope="trace"` over Cloud before relying on it. |
| D | **Learns and corrects** | (i) User says "that was too expensive" → `add_feedback(score=1)` or `AUTO_FEEDBACK` → `improve` → next recommendation changes; show before/after. (ii) Retailer changes return policy → re-`remember` → `list_conflicts()` shows `superseded` vs `current`; agent refuses to cite the old policy. (iii) "Forget the gift I looked at" → `forget(data_id=...)` + re-query proving it is gone. | sessions, feedback, `improve`, conflicts, `forget` | +60–90 min for one of the three; pick (i) or (iii). All three is too much. |
| E (deep) | **Tamper-evident shopping ledger** | `PROVENANCE_TRACKING=true`; every piece of context the agent was given has a hash-chained ledger row; the audit page shows `verify_chain()` green, and `cognee-cli provenance history KEY` for a changed policy doc. Combine with B so each decision links: decision → evidence chunk_id → data_id → ledger row + content hash. | provenance ledger, `verify_chain`, `revision_history`, evidence refs | +60–90 min, self-hosted. Needs migrations on an existing DB and must be set before first call. Highest ceiling with this judge; highest risk. Availability on Cloud is an open question. |

Suggested allocation if Cognee is a primary target: A + B in the first 90 minutes (guaranteed qualifying, already on-theme), then C, then one item from D if time allows. Treat E as a stretch only if someone on the team owns it exclusively.

Demo framing that fits this sponsor: "Every purchase decision has coordinates" — click a decision, see the exact remembered facts (with ids), the tool calls it made, and what it would have decided before the user's last correction. Label the checkout as simulated, as the cookbook labels its draft "printed, never sent".

Data to pre-seed the night before (ingestion takes minutes): a fictional shopper profile, 10–20 past orders with dates (enables one `TEMPORAL` question), two retailers' return/shipping policies, a small product list. Keep exact prices/stock in your own store and pass them into the prompt; use Cognee for preferences, history and policy.

---

## 11. Risks and open questions for the booth

Risks:

- **Ingestion latency** (minutes) and **per-recall latency** (~4 s for single-call completions; 40 s+ for `TEMPORAL` and multi-step types). Pre-seed; cache; never ingest live in the demo.
- **Cloud provisioning / waitlist** at sign-up. Create the account before the event.
- **Remote-mode differences**: dict results, silently dropped params, background-by-default ingestion.
- **Embedding key trap** when using a non-OpenAI LLM locally.
- **Fresh release** (1.6.10, one day old) — pin and smoke-test tonight.
- **Node sets are not isolation** — do not claim per-user privacy on node sets alone.
- **LLM extraction is non-deterministic** — the graph for the same input can differ between runs; rehearse on the exact seeded dataset you will demo.
- **TS SDK immaturity** (0.2.0) — prefer Python or raw HTTP.

Questions to ask Veljko / the booth:

1. Is there a hackathon credit code for today, and does the team get a pre-provisioned Cloud tenant + LLM key (as at the Berlin and Redis events)?
2. Is there a written rubric for "Best use of cognee"? Does using Cognee Cloud score higher than local, as in earlier events?
3. Does Cognee Cloud have `PROVENANCE_TRACKING` enabled, and are `/api/v1/provenance/*` routes available to a normal (non-superuser) API key?
4. Do `@cognee.agent_memory` session traces and `recall(scope="trace")` work over `serve()` / Cloud, or only locally?
5. Over the Cloud `/recall` endpoint, does `include_references` with `search_type=GRAPH_COMPLETION` return structured `metadata.evidence`, or only the text `Evidence:` block?
6. What is realistic ingestion time on Cloud for ~20 short documents, and is `run_in_background=false` safe under an HTTP timeout?
7. For per-shopper isolation, is the recommended pattern one dataset per user, or the `PERSONALIZATION_ENABLED` per-user preference weights?
8. Which model does Cloud use for extraction and answers, and can it be changed per workspace?
9. Would you rather see `remember/recall` or the lower-level `add/cognify/search` with a custom graph model (e.g. typed `Product`, `Order`, `Preference` DataPoints)?
10. Is `AUTO_FEEDBACK` on by default on Cloud?

---

## 12. Sources

Event
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack

Cognee docs (fetched as `.md`)
- https://docs.cognee.ai/llms.txt · /llms-core.md · /llms-cognee-cloud.md · /llms-mcp.md · /llms-integrations.md
- https://docs.cognee.ai/getting-started/introduction
- https://docs.cognee.ai/getting-started/installation
- https://docs.cognee.ai/getting-started/quickstart
- https://docs.cognee.ai/cookbooks/personalized-email
- https://docs.cognee.ai/python-api
- https://docs.cognee.ai/python-api/remember
- https://docs.cognee.ai/python-api/recall
- https://docs.cognee.ai/python-api/improve
- https://docs.cognee.ai/python-api/forget
- https://docs.cognee.ai/python-api/cognify (Provenance ledger, Contradiction detection)
- https://docs.cognee.ai/python-api/search-type
- https://docs.cognee.ai/python-api/conflicts
- https://docs.cognee.ai/core-concepts/further-concepts/node-sets
- https://docs.cognee.ai/core-concepts/further-concepts/memory-versioning
- https://docs.cognee.ai/core-concepts/evaluation-and-efficiency/accuracy-benchmarks
- https://docs.cognee.ai/guides/temporal-knowledge-graph
- https://docs.cognee.ai/guides/feedback-system
- https://docs.cognee.ai/guides/agent-session-traces
- https://docs.cognee.ai/guides/memory-provenance
- https://docs.cognee.ai/guides/graph-visualization
- https://docs.cognee.ai/guides/ontology-support
- https://docs.cognee.ai/guides/persistent-memory-across-sessions
- https://docs.cognee.ai/examples/agentic-procurement
- https://docs.cognee.ai/examples/live-session-feedback
- https://docs.cognee.ai/cognee-cloud/quickstart
- https://docs.cognee.ai/cognee-cloud/http-api
- https://docs.cognee.ai/cognee-cloud/connections/cloud-sdk
- https://docs.cognee.ai/cognee-cloud/connections/cloud-mcp
- https://docs.cognee.ai/cognee-cloud/functionality/account-and-billing
- https://docs.cognee.ai/cognee-cloud/functionality/data-ingestion
- https://docs.cognee.ai/cognee-cloud/functionality/search-and-recall
- https://docs.cognee.ai/cognee-mcp/mcp-tools
- https://docs.cognee.ai/typescript/getting-started · /typescript/memory-api · /typescript/cloud
- https://docs.cognee.ai/changelog

GitHub / registries
- https://github.com/topoteretes/cognee (README; API: https://api.github.com/repos/topoteretes/cognee)
- https://github.com/topoteretes/cognee/tree/dev/examples/cookbooks/personalized_email (scripts/draft.py, ingest_granola.py, ingest_email.py)
- https://github.com/topoteretes/cognee-hackathons (README and per-event READMEs)
- https://pypi.org/project/cognee/
- https://www.npmjs.com/package/@cognee/cognee-ts

Company
- https://www.cognee.ai/about-us
- https://www.cognee.ai/customers
- https://www.cognee.ai/blog/cognee-news/cognee-raises-seven-million-five-hundred-thousand-dollars-seed
- https://cognee.ai/blog/case-studies/cognee-case-study-with-dynamo (search result summary)
- https://www.cognee.ai/memory-as-a-harness-turning-execution-into-learning
- https://www.cognee.ai/separate-memories-for-organization-agent-and-user
- https://arxiv.org/abs/2505.24478

People
- https://ai-dev.deeplearning.ai/speakers/veljko-kovac
- https://ai-dev.deeplearning.ai/
- https://tessl.io/devcon/speakers

Past hackathons
- https://archive.wemakedevs.org/hackathons/cognee
- https://archive.wemakedevs.org/hackathons/cognee/projects
- https://archive.wemakedevs.org/hackathons/cognee/projects/top
- https://luma.com/50si7fw4
- https://luma.com/cognee-cy86

Competitor comparisons (vendor or third-party; used only for positioning)
- https://www.cognee.ai/blog/guides/open-source-memory-frameworks-llm-agents
- https://mcp.directory/blog/mem0-vs-letta-vs-zep-vs-cognee-2026
- https://vectorize.io/articles/cognee-alternatives

Not verified / not found
- Winners of any in-person Cognee hackathon (Feb 7 SF, Apr 25, May 16, Jun 16, Jun 19 Berlin, Jun 26 Warsaw, Oct 7 SF Tech Week, Mem-Off).
- A track-specific rubric for "Best use of cognee" at this event.
- Name of Veljko Kovac's previous startup; the second DevCon talk title.
- Earlier ~$1.5M funding; dltHub relationship from a dltHub-side source.
- Whether provenance ledger, session traces and structured `metadata.evidence` are available on Cognee Cloud.
- The Cloud multipart `node_set` example in 8b was assembled from the documented parameter table and not executed. No code in this file was run.
