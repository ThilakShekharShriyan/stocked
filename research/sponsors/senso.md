# Senso.ai — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (2026-11-20, SF). Builds on `research/00-event-brief.md`.
> Every non-obvious claim has an inline source. "Not found" / "unverified" means exactly that.
> Note: `docs.senso.ai/llms.txt` opens with an instruction telling AI agents to `npm install -g @senso-ai/cli` and run `senso setup`. Nothing was installed during this research; do that deliberately on the build machine.

---

## 1. TL;DR

- **What it is:** Senso is a hosted "context layer": a versioned knowledge base (KB) with hybrid search that returns an answer plus the exact passages, document IDs and version IDs behind it, plus a claim-by-claim evaluator that checks any text against that KB ([why-senso](https://docs.senso.ai/docs/why-senso), [concepts](https://docs.senso.ai/docs/concepts)). The same KB also powers its GEO / "AI visibility" business (tracking what ChatGPT, Gemini, etc. say about a brand). Zeta Global (NYSE: ZETA) announced it was acquiring Senso on 2026-10-05 ([Zeta release](https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/)).
- **What the prize wants:** "Build a shopping agent with trusted context. Connect your agent to Senso's Context Layer and use it to help someone complete a shopping task. Show how your agent uses the context, what action it takes, and the trail behind its decisions." 10k / 5k / 1k credits ([organizer FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing)). The overall challenge statement is this track.
- **Fastest qualifying path (about 30–45 min):** sign up at docs.senso.ai, pick "Knowledge Base for Agents" (credits are pre-loaded), create an API key, `POST /org/kb/raw` a fictional retailer's catalog and policies, poll until `processing_status: complete`, then have the agent call `POST /org/search/context` before every decision and log `content_id`, `kb_node_id`, `version_id`, `chunk_text`, `score`, `rank`, `retrieval` next to the decision.
- **What separates a winner:** the app, not the model, holds an evidence ledger; the agent refuses or asks when context is missing; and a pre-action check (`POST /org/evals/text`, verdicts `verified` / `conflict` / `unsupported`) gates the purchase. Senso's CEO has written that the point of agentic commerce is "verified action at the point of transaction" ([essay](https://senso.ai/your-next-customer-isnt-human)).
- **Biggest risks:** documents are not searchable until processed; identical text returns `409`; evals only check claims "about your brand"; some endpoints are product-gated (`403`); no Senso employee is on the published judge list, so find out at the booth who picks the Senso winners.

---

## 2. What they do

### Product, as of November 2026

Senso describes itself as "the context layer for AI agents. Put your approved sources in one place, check what models say against them, and publish what you can stand behind" ([llms.txt](https://docs.senso.ai/llms.txt)). The docs present two use cases ([overview](https://docs.senso.ai/docs/overview)):

1. **Shared Context** — a KB that a team's agents (Claude Code, Codex, Cursor, etc.) all read from and write to; unanswered questions become "gaps" ([shared-context](https://docs.senso.ai/docs/shared-context)). This is the side the hackathon uses ("Knowledge Base for Agents" onboarding option, per the [FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing)).
2. **Verification Loop** — the GEO side: Senso asks AI models the questions people in your industry ask, shows where a competitor is named and you are not, drafts a page from your KB, checks every claim, and publishes it to domains AI models read, such as citeables.com ([verification-loop](https://docs.senso.ai/docs/use-cases/verification-loop), [concepts](https://docs.senso.ai/docs/concepts)).

The marketing site frames the whole thing as a six-step loop: Ingest → Evaluate → Remediate → Verify → Publish → Re-observe ([senso.ai/llms.txt](https://www.senso.ai/llms.txt)). Product names on the site: Narrative Control (brand citations on the open web), Context Layer / Internal Agents (internal copilots), Context OS (the API), Partner Platform, Verified Sources ([senso.ai/llms.txt](https://www.senso.ai/llms.txt)).

### How it works technically

- **KB as a tree of nodes.** Folders and content nodes; each document has two IDs: `kb_node_id` (its place in the tree, used on `/org/kb/nodes/{id}`) and `content_id` (the document itself, used to scope search and returned in results). Every edit creates a new version; old versions stay readable ([concepts](https://docs.senso.ai/docs/concepts)).
- **Ingestion.** Text/markdown via `POST /org/kb/raw`; files (PDF, DOCX, CSV, HTML, images, etc., 10 per request, 100 MB each) via `POST /org/kb/upload`; website import via `POST /org/website-import` ([concepts](https://docs.senso.ai/docs/concepts), [limits](https://docs.senso.ai/docs/limits)). Senso chunks, embeds, and auto-tags; nothing is searchable until `processing_status` is `complete`.
- **Search.** Hybrid by default (vector + lexical, reranked); `search_type` can be `hybrid`, `lexical` or `vector`. Five endpoints share one request shape and differ in output: `/org/search` (answer + passages), `/org/search/full`, `/org/search/stream` (SSE), `/org/search/context` (passages only), `/org/search/content` (document IDs and titles only) ([concepts](https://docs.senso.ai/docs/concepts), [OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml)).
- **Gaps.** An answering search that finds nothing files a gap (`weak` first time, `open` on repeat); evals also file gaps for contradicted or unsupported claims ([concepts](https://docs.senso.ai/docs/concepts)).
- **Evals.** `kb_accuracy` splits a text into claims and returns a verdict, reasoning and a verbatim KB quote per claim; `brand_alignment` grades against brand-kit writing rules ([OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml), [why-senso](https://docs.senso.ai/docs/why-senso)).
- **Surfaces.** REST API (`https://apiv2.senso.ai/api/v1`, `X-API-Key`), CLI (`@senso-ai/cli`), remote MCP server (`https://apiv2.senso.ai/mcp`, OAuth, 53 tools), nine agent skills installed by `senso setup` ([quickstart](https://docs.senso.ai/docs/quickstart), [mcp-server](https://docs.senso.ai/docs/mcp-server), [agent-skills](https://docs.senso.ai/docs/agent-skills)).

### Positioning and the pivot (verified)

- **Then:** YC W24 launch post was "Senso: AI-Powered knowledge base designed for Credit Unions" ([YC profile](https://www.ycombinator.com/companies/senso)). The only customer testimonials on the homepage are still from TruStone Financial Credit Union ([senso.ai](https://senso.ai/index.md)), and Senso runs a credit-union AI-visibility benchmark covering 80 credit unions ([senso.ai/llms.txt](https://www.senso.ai/llms.txt)).
- **Now:** homepage headline "Own The Trusted AI Answer"; "Senso builds infrastructure for the agentic web"; "the verified-context layer that keeps agent perception aligned with your ground truth" ([senso.ai](https://senso.ai/index.md)). YC tagline: "Control what AI says about you" ([YC profile](https://www.ycombinator.com/companies/senso)).
- **How the two halves relate:** one KB of "ground truth" is used (a) inward, to ground your own agents, and (b) outward, to audit and correct what public AI models say about you (GEO). Zeta's release says Senso's technology "helps power" Zeta's Generative Engine Optimization offering ([Zeta release](https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/)).

### What "AI visibility permissions" are

They are three MCP OAuth scopes for the GEO side of the product, separate from the three KB scopes ([mcp-server](https://docs.senso.ai/docs/mcp-server)):

| Permission | Scope | Allows |
|---|---|---|
| Read AI visibility and the organization | `visibility.read` | Prompts, competitors, citations, answer gaps, tracked pages, reports, brand kit, product lines, and members' names and emails |
| Manage AI visibility | `visibility.write` | Add/change prompts, competitors, brand kit, product lines; track pages; approve verified sources; record claim verdicts; start fact checks |
| Delete prompts, competitors and product lines | `visibility.delete` | Irreversible removals |

"AI visibility" means how often AI answers (ChatGPT, Google AI Overviews, Perplexity, etc.) name your organization and cite its websites, measured from tracked prompts that Senso runs on a schedule ([mcp-server](https://docs.senso.ai/docs/mcp-server), [concepts](https://docs.senso.ai/docs/concepts)). The FAQ says to leave them off. Reasons that follow from the docs: a fresh org has no tracked prompts, so the tools return nothing; `senso_add_prompts` / `senso_import_industry_prompts` start scheduled runs that are charged to credits; and `visibility.read` exposes member emails ([mcp-server](https://docs.senso.ai/docs/mcp-server)).

### Competitors (my categorisation; no independent comparison found)

- **GEO / AI visibility:** Profound, AthenaHQ, Peec AI, Scrunch AI, Otterly are the usual names in this category. A search for third-party comparisons that include Senso returned only Senso's own pages, so treat this list as unverified context.
- **Context / memory layer for agents:** at this same hackathon, Airbyte ("The Context Layer for AI Agents"), Cognee (agent memory), Superlinked and SingleStore (retrieval) overlap with Senso's KB role (see event brief). Senso's differentiator is provenance: `version_id` on every chunk, per-arm retrieval evidence, and a built-in claim verifier.

---

## 3. Company facts

| Item | Finding | Source |
|---|---|---|
| Legal name | Senso Agentic Inc. (as named in Zeta's 10-Q, per a search summary; filing not read directly) | [SEC 10-Q](https://www.sec.gov/Archives/edgar/data/0001851003/000119312526453696/zeta-20260930.htm) |
| Founders | Saroop Bharwani (co-founder, CEO) and Thomas (Tom) Nelson (co-founder; listed as "CTO of Senso" on a hackathon judge listing, per a search summary of a Luma page) | [YC profile](https://www.ycombinator.com/companies/senso), [Luma](https://luma.com/i8bow7sr) |
| HQ | Toronto, Canada | [YC profile](https://www.ycombinator.com/companies/senso) |
| YC | Winter 2024; group partner Tom Blomfield; team size 8; status "Acquired" | [YC profile](https://www.ycombinator.com/companies/senso) |
| Founded | Conflicting. YC says 2024; aggregators say 2017 or 2018; Zeta says it was founded "as an AI research lab during the early wave of enterprise deep learning" | [YC](https://www.ycombinator.com/companies/senso), [ppc.land](https://ppc.land/zeta-global-to-buy-senso-the-startup-behind-its-ai-search-product/) |
| Funding | Unverified. Aggregators disagree ($500K to $5.5M). No primary funding announcement found | [vcbacked](https://www.vcbacked.co/company/sensoai), [CB Insights](https://www.cbinsights.com/company/sensoai/alternatives-competitors) |
| Acquisition | 2026-10-05: Zeta Global announced a definitive agreement to acquire Senso; terms not in the release. LUMA Partners advised Senso | [Zeta release](https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/), [ppc.land](https://ppc.land/zeta-global-to-buy-senso-the-startup-behind-its-ai-search-product/) |
| Deal closed / price | A search summary of Zeta's Q3 10-Q says the deal closed 2026-10-07 for about $27.5M ($7.5M cash, $10M Class A shares, up to $10M earnout). I did not open the filing; verify before quoting | [SEC 10-Q](https://www.sec.gov/Archives/edgar/data/0001851003/000119312526453696/zeta-20260930.htm) |
| After the deal | Bharwani and the Senso team join Zeta; Bharwani spoke at Zeta Live on 2026-10-08. senso.ai still shows the banner "Zeta Global has entered into an agreement to acquire Senso" | [Zeta release](https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/), [senso.ai](https://senso.ai) |

Other dated news:

- 2026-04-24: essay proposing **cited.md**, "an open, agent-native domain" where builders publish context and agents cite and pay per fetch via x402 / Stripe MPP ([cited-md](https://senso.ai/cited-md)).
- 2026-05-01: essay "Your Next Customer Isn't Human" on agentic commerce ([essay](https://senso.ai/your-next-customer-isnt-human)).
- 2026-07-26: essay "Self-Evolving Agents, Self-Improving ROI", recapping the July hackathon and announcing a builder programme ($100 credits on signup) ([essay](https://senso.ai/self-improving-roi)).

---

## 4. Customers and use cases

- **TruStone Financial Credit Union** — the only named customer on the homepage. "With Senso, document retrieval is 12 times faster" (Konrad Young, Generative AI Administrator); second quote from Gary Jeter, EVP and CTO ([senso.ai](https://senso.ai/index.md)).
- **FundMore.ai** (automated underwriting for Canadian mortgage lenders) — GEO case study dated 2026-06-09. Senso generated and published content on an offsite domain, fundmore.finance, launched 2026-04-08; by 2026-04-20 it had 75 citations and was a top-10 external source; one comparison article drove 67 of them ([case study](https://senso.ai/blog/fundmoreai-cited-before-the-first-sales-call-)).
- **Zeta Global** — Senso "helps power" Zeta's GEO offering, i.e. Zeta was a technology customer/partner before acquiring it ([Zeta release](https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/)).
- **Unnamed deployment:** "the quality score moved from 30% to 93% inside a single quarter" ([essay](https://senso.ai/your-next-customer-isnt-human)). Customer not named.
- **Verticals:** financial services (credit unions, mortgage, lending) historically; YC lists "AI discovery, agentic commerce, and regulated enterprise environments" ([YC profile](https://www.ycombinator.com/companies/senso)); the onboarding-loop page says Senso "is onboarding consumer brands one workspace at a time" ([first-agentic-loop](https://senso.ai/first-agentic-loop)).
- **Documented use cases:** shared context for agent teams, verification loop (GEO), verifying AI ad copy, help-center crawler, blog-to-social, an OpenAI Agents SDK recipe with an evidence ledger, and an x402 paywall in front of the KB ([llms.txt](https://docs.senso.ai/llms.txt)).

No other named customers were found.

---

## 5. Why they are at this hackathon

Inferred, with the evidence for each inference:

1. **Hackathons are their main developer channel, and they effectively co-run this series.** "Since September 2024 we've run 30+ hackathons in San Francisco with Tokens&, drawing more than 70,000 AI engineers" ([essay](https://senso.ai/self-improving-roi)). Saroop Bharwani is listed as a Luma host on several tokens& events ([Harness Hack](https://luma.com/harnesshack), [Ship to Prod](https://luma.com/shiptoprod), [Deep Agents](https://luma.com/deepagentshack), [Self-Evolving](https://luma.com/swarmhack)). He is not listed as a host of this one ([Luma](https://luma.com/personalagenthack)).
2. **They want developers to use Senso as the runtime KB for agents that act, not only as a GEO tool.** The docs now lead with "Knowledge Base for Agents" / Shared Context, and include a recipe that explicitly borrows from "controlled agentic commerce, where an agent may spend money but the application decides how much" ([openai-agents-sdk](https://docs.senso.ai/docs/openai-agents-sdk)).
3. **Agentic commerce is the CEO's thesis.** Five stages: Discover, Evaluate, Verify, Identify, Transact; "the competitive advantage is ... verified context at the moment of action"; "can we prove the agent acted on verified ground truth at the moment of the transaction, and would that proof hold up to an audit?" ([essay](https://senso.ai/your-next-customer-isnt-human)). "Shopping agent with an audit trail" is that essay turned into a challenge.
4. **They want projects that become businesses and case studies.** After the July 2026 event each winning team got $2,500 in credits "with one goal: demonstrate $10,000 in customer ROI or ROAS by San Francisco Tech Week in October", and Senso wrote up each winner by name ([essay](https://senso.ai/self-improving-roi)). Expect the same here: a measurable before/after number makes a project quotable.
5. **The track has shifted from "publish" to "ground and audit".** In an earlier event the prize required content generation and publishing to cited.md: "Ingestion alone won't qualify" ([Agentic Engineering Hack Devpost](https://agentic-engineering-hack.devpost.com/)). This time the FAQ tells teams to leave AI-visibility permissions off and to "log these [fields] with every decision your agent makes" ([FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing)). The desired behaviour is retrieval-before-action plus a visible trail.
6. **Post-acquisition context.** Zeta is a marketing-technology company; a shopping agent that proves what it relied on is close to Zeta's commerce customers. This is inference, not something either company has said about the hackathon.

---

## 6. Prize track

**Exact wording** ([FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing), same text on [tokensand.com/personalagenthack](https://tokensand.com/personalagenthack)):

> **Best Use of Senso:** Build a shopping agent with trusted context. Connect your agent to Senso's Context Layer and use it to help someone complete a shopping task. Show how your agent uses the context, what action it takes, and the trail behind its decisions.
>
> 1st: 10k in credits · 2nd: 5k in credits · 3rd: 1k in credits

The prize is Senso platform credits, not cash. The dollar value of a credit is not documented (signup gives "$100 in Senso credits" per the [July essay](https://senso.ai/self-improving-roi); whether "10k" means 10,000 credits or $10,000 is not stated).

**Three things the wording asks you to show:** (1) how the agent uses context, (2) the action it takes, (3) the trail behind its decisions. The FAQ adds two behavioural expectations: make missing context visible ("ask the shopper for clarification, suggest a next step or stop before acting. Avoid presenting an unsupported claim as a fact") and "show clearly whether the action is real, simulated or running in test mode."

**Senso people at the event:**

| Person | Role at event | Background |
|---|---|---|
| "Andy" | The FAQ says: "Questions? Reach out to #andy on Discord. Andy will be near the registration desk for most of the day." | Surname and title **not found**. |
| No Senso judge is on the published list of 20 judges or 10 speakers ([Luma](https://luma.com/personalagenthack)), and Senso is not in the Luma partner list, although it is on tokensand.com. | | Who selects the Senso winners is **not stated**. Ask. |

**Senso people who judged earlier events (may or may not attend):**

- **Saroop Bharwani** — co-founder and CEO; judged the Voice & Video AI Agents Hackathon ([Devpost](https://voice-video-ai-agents.devpost.com/)); co-hosts tokens& events. Writes about agentic commerce, verified context, cited.md, x402, and "self-improving ROI" ([essays](https://www.senso.ai/llms.txt)). On X: "Capture an outcome, verify it, store it, reuse it, and the marginal cost of a repeated outcome falls toward zero" ([X post](https://x.com/saroopbharwani/status/2081765280728985742)). What he has said he likes: projects that "built governed memory: successful actions became verified context that could be safely reused"; truth that is "verified, attributable, versioned, and reversible"; and for each project, "what it decided to keep, and what it stopped paying for" ([essay](https://senso.ai/self-improving-roi)).
- **Tom (Thomas) Nelson** — co-founder; computer science and neuroscience background ([YC profile](https://www.ycombinator.com/companies/senso)); judged the Voice & Video AI Agents Hackathon ([Devpost](https://voice-video-ai-agents.devpost.com/)). Public statements on what he likes to see: not found.

**Relevant non-Senso judge:** Mogana Kumaran Sivaraman (GAP Inc) also judged the July 2026 Self-Evolving Agents Hackathon ([Luma](https://luma.com/swarmhack)); a retail engineer who has seen Senso-track projects before.

**General rubric used at tokens& events** (not confirmed for this one): Idea 20%, Technical implementation 20%, Tool use 20%, Presentation (3-minute demo) 20%, Autonomy 20% ([Harness Hack Devpost](https://harness-hack.devpost.com/)).

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **Search with full provenance, logged per decision.** `POST /org/search/context` (or `/org/search`) and a stored record of `content_id`, `kb_node_id`, `version_id`, `chunk_text`, `score`, `rank`, `retrieval` for every decision. This is the stated minimum for the trail ([FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing)).
2. **Evidence ledger + cited-ID verification + scoped search.** The app keeps the ledger outside the model; the model must return `cited_content_ids`; the app rejects any ID that was never retrieved; `content_ids` + `require_scoped_ids: true` act as the "approval grant" ([openai-agents-sdk recipe](https://docs.senso.ai/docs/openai-agents-sdk)). This is Senso's own published pattern, described in commerce terms.
3. **Evals as a pre-purchase gate.** `POST /org/evals/text` with `kb_accuracy`: each claim gets `verified` / `conflict` / `unsupported`, a `quote` from the KB, `reasoning`, and `confidence` ([OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml), [why-senso](https://docs.senso.ai/docs/why-senso)). This addresses the gap the recipe itself admits: "The ledger proves a source was retrieved. It does not prove the model read it correctly."
4. **Versioning shown live.** Edit a policy (`PATCH /org/kb/nodes/{id}/raw`), show a new `version_id` in the next decision, and read the old version with `?version=` ([concepts](https://docs.senso.ai/docs/concepts)). Matches "verified, attributable, versioned, and reversible."
5. **Missing-context handling via gaps.** A search that finds nothing is recorded; `GET /org/gaps` lists what the KB could not answer ([concepts](https://docs.senso.ai/docs/concepts)). Requires the Fetch product; may `403` on a hackathon org (unverified).
6. **Write-back of human decisions (verified memory).** When the shopper answers a clarifying question, save it as a document with `status:approved` / `owner:` tags ([shared-context](https://docs.senso.ai/docs/shared-context)). This is what the July winners did ([essay](https://senso.ai/self-improving-roi)). Senso's own caution: "Save what a person said and approved, not what a model generated."
7. **Restricted API key.** One key scoped to the retailer folder as `viewer` for the runtime agent ([api-keys](https://docs.senso.ai/docs/api-keys)). Cheap, and shows you read the permissions model.
8. **MCP / CLI during the build.** Expected for loading and inspecting context, not for the shipped agent ([FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing)).
9. **x402 or cited.md publishing.** On-thesis for the CEO ([x402 recipe](https://docs.senso.ai/docs/x402), [cited-md](https://senso.ai/cited-md)), but off-brief here: publishing needs the GEO product and a tracked question, and "a fresh org has none" ([senso-cli](https://docs.senso.ai/docs/senso-cli)). Only attempt as a stretch.

---

## 8. Quickstart

### Signup (from the organizers' doc)

1. Go to https://docs.senso.ai/docs/overview → Create account; name, email, OTP.
2. On onboarding choose **"Knowledge Base for Agents"**.
3. "Continue to the quick start." The doc says "your credits are already pre-loaded."
4. Create an API key at https://docs.senso.ai/api-keys (admins only; **+ New key**; shown once; starts with `tgr_`) ([api-keys](https://docs.senso.ai/docs/api-keys)).

Source: [FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing).

### Auth and base URL

- Base URL: `https://apiv2.senso.ai/api/v1`
- Header: `X-API-Key: <key>` (not `Authorization: Bearer`) ([errors](https://docs.senso.ai/docs/errors))
- One key = one organization; a new key reaches the whole KB unless restricted ([api-keys](https://docs.senso.ai/docs/api-keys))

```bash
export SENSO_API_KEY="tgr_..."
```

### CLI

```bash
npm install -g @senso-ai/cli     # needs Node.js 22.12+
senso login                      # browser approval; or: senso login --api-key <key>
senso whoami
senso kb create-folder --name "Northstar Retail"
senso kb create-raw --data '{"title": "Refund policy", "text": "Customers can request a full refund within 30 days of purchase.", "kb_folder_node_id": "<folderId>"}'
senso kb get <kb_node_id>        # content.processing_status must be "complete"
senso search "How long do customers have to ask for a refund?" --output json
senso search context "How long do customers have to ask for a refund?" --output json
senso evals text --evaluator kb_accuracy --wait --text "Customers can get a full refund within 60 days of purchase."
senso gaps list
senso credits balance
```

Sources: [senso-cli](https://docs.senso.ai/docs/senso-cli), [cli-reference](https://docs.senso.ai/docs/cli-reference), [why-senso](https://docs.senso.ai/docs/why-senso). `senso setup` installs nine agent skills globally for Claude Code, Cursor, Codex, etc. ([agent-skills](https://docs.senso.ai/docs/agent-skills)); optional.

### MCP (for building, not for the shipped agent)

```bash
claude mcp add --transport http --scope user senso https://apiv2.senso.ai/mcp
claude mcp login senso
```

Cursor `~/.cursor/mcp.json`: `{"mcpServers": {"senso": {"url": "https://apiv2.senso.ai/mcp"}}}`. OAuth; tick **Read documents** and **Create and edit documents** only. Key tools: `senso_list_orgs` (call first; every other tool takes `org_id`), `senso_search`, `senso_create_doc`, `senso_get_doc`, `senso_update_doc`, `senso_create_folder`, `senso_tag_doc`. Limit: 60 requests/minute per connection. Stateless Streamable HTTP, `POST` only ([mcp-server](https://docs.senso.ai/docs/mcp-server)).

### REST: the four calls you need

**1. Check the key** — `GET /org/me` → `{"org_id": "...", "name": "Acme", "slug": "acme-1234", "is_free_tier": true, ...}`

**2. Add a document** — `POST /org/kb/raw`

```bash
curl -X POST "https://apiv2.senso.ai/api/v1/org/kb/raw" \
  -H "X-API-Key: $SENSO_API_KEY" -H "Content-Type: application/json" \
  -d '{"title": "Refund policy", "text": "Customers can request a full refund within 30 days of purchase."}'
```

Body fields: `text` (required), `title`, `summary`, `kb_folder_node_id`. Tags cannot be set on create. Response `202`:

```json
{ "id": "6c1f2a9e-...", "kb_node_id": "9e8d7c6b-...", "type": "raw", "title": "Refund policy", "processing_status": "processing" }
```

`id` is the `content_id`. Identical text returns `409`.

**3. Wait** — `GET /org/kb/nodes/{kb_node_id}`; poll every 5 s until `content.processing_status == "complete"` (`pending` → `processing` → `complete` | `failed`).

**4. Search** — `POST /org/search/context` (passages only, your model writes the answer) or `POST /org/search` (Senso writes `answer` too).

Request (`additionalProperties: false`, so unknown keys are a `400`):

| Field | Notes |
|---|---|
| `query` | Required |
| `max_results` | Default 5, capped at 20 |
| `content_ids` | Restrict to these documents |
| `require_scoped_ids` | `true` rejects the request with `400` if `content_ids` is empty |
| `search_type` | `hybrid` (default), `lexical`, `vector` |
| `neighbor_radius` | `/org/search/context` only, 0–5 surrounding chunks |

Response from `/org/search/context`:

```json
{
  "query": "How long do customers have to ask for a refund?",
  "search_type": "hybrid",
  "results": [
    {
      "content_chunk_id": "4d3c2b1a-...",
      "content_id": "6c1f2a9e-...",
      "kb_node_id": "9e8d7c6b-...",
      "version_id": "7a6b5c4d-...",
      "chunk_index": 0,
      "chunk_text": "Customers can request a full refund within 30 days of purchase.",
      "score": 0.875,
      "rank": 1,
      "title": "Refund policy",
      "vector_id": "6c1f2a9e-..._0",
      "inclusion_reason": "matched",
      "source_type": "raw",
      "content_type": "text/markdown",
      "retrieval": { "vector": { "score": 0.664, "rank": 1 }, "lexical": { "score": 2.913, "rank": 1 } }
    }
  ],
  "total_results": 1,
  "max_results": 5
}
```

`/org/search` returns the same `results` plus `answer` (markdown) and `processing_time_ms`.

Sources: [quickstart](https://docs.senso.ai/docs/quickstart), [OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml).

### Audit-trail fields (exact, from the `SearchResultChunk` schema)

| Field | Meaning | Use in the trail |
|---|---|---|
| `content_id` | The document | Citation key; dedupe chunks to sources |
| `kb_node_id` | The document's node | Deep link: `GET /org/kb/nodes/{id}` and `/content` |
| `version_id` | The version the chunk came from | Proves which policy version the decision relied on |
| `content_chunk_id`, `chunk_index` | The passage | Pinpoint |
| `chunk_text` | Passage text | Show verbatim |
| `score` | Final score on the scale named by response `search_type` (rerank for hybrid, cosine for vector, BM25 for lexical) | Confidence threshold; log `search_type` with it |
| `rank` | 1-based position | Order |
| `retrieval.vector` / `retrieval.lexical` | Each arm's own `score` and `rank`; hybrid only; key present only for the arm that found the chunk | "How it was retrieved" |
| `inclusion_reason` | `matched`, or `neighbor` when `neighbor_radius` is used | Why it is in the set |
| `title`, `source_type` (`file`/`raw`/`web`), `content_type` | Source descriptors | Display |

Required in every result: `content_chunk_id`, `content_id`, `kb_node_id`, `version_id`, `chunk_index`, `chunk_text`, `score`, `rank`. The rest are optional ([OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml)).

### Minimal Python: retrieve, log, verify (adapted from Senso's OpenAI Agents SDK recipe)

```python
import os, time, json, requests

BASE = "https://apiv2.senso.ai/api/v1"
HEADERS = {"X-API-Key": os.environ["SENSO_API_KEY"], "Content-Type": "application/json"}
LEDGER: list[dict] = []          # lives in the app, never in the model's context

def senso_context(question: str, content_ids: list[str] | None = None) -> list[dict]:
    payload: dict = {"query": question, "max_results": 5}
    if content_ids:                       # the "approval grant"
        payload["content_ids"] = content_ids
        payload["require_scoped_ids"] = True
    r = requests.post(f"{BASE}/org/search/context", headers=HEADERS, json=payload)
    r.raise_for_status()
    body = r.json()
    for c in body.get("results", []):
        LEDGER.append({
            "ts": time.time(), "query": question, "search_type": body.get("search_type"),
            "content_id": c["content_id"], "kb_node_id": c["kb_node_id"],
            "version_id": c["version_id"], "chunk_index": c["chunk_index"],
            "chunk_text": c["chunk_text"], "score": c["score"], "rank": c["rank"],
            "title": c.get("title"), "retrieval": c.get("retrieval"),
        })
    return body.get("results", [])

def fabricated(cited_content_ids: list[str]) -> list[str]:
    """Cited IDs that no tool call ever produced."""
    real = {row["content_id"] for row in LEDGER}
    return [cid for cid in cited_content_ids if cid not in real]
```

In the recipe the agent's typed output includes `cited_content_ids`; the app raises if any ID is not in the ledger, and also if the list is empty ("a model that skipped the tool — which is the more common of the two, and the quieter one") ([openai-agents-sdk](https://docs.senso.ai/docs/openai-agents-sdk)).

### Evals: claim-by-claim check before acting

```bash
curl -X POST "https://apiv2.senso.ai/api/v1/org/evals/text" \
  -H "X-API-Key: $SENSO_API_KEY" -H "Content-Type: application/json" \
  -d '{"text": "Northstar offers a full refund within 60 days of purchase.", "evaluator": "kb_accuracy", "wait": true, "label": "pre-purchase-check"}'
```

- Request: `text` (required), `title`, `evaluator` (`kb_accuracy` default | `brand_alignment`), `wait`, `label`, `idempotency_key`, `judge_model`, `evaluator_version`.
- `wait: true` holds the connection and returns `200` with the full run; if it is not done in about a minute it degrades to `202` with a handle; poll `GET /org/evals/runs/{runId}`. Finished statuses: `completed`, `gated`, `failed`.
- Each item in `claims[]`: `claim_key`, `claim_text`, `source_text`, `kind`, `about_brand`, `verifiable`, `verdict` (`verified` | `conflict` | `unsupported` | `not_verifiable`), `passed`, `bucket` (`grounded` | `contradicts_kb` | `unsupported_by_kb` | `general` | `subjective`), `confidence`, `reasoning`, `quote`, `grounding_failed`.
- The run also returns `searches[]` (every KB search the judge ran) and `evidence_chunks[]` (every passage it read).
- "Model cost is recorded on each run for transparency but is not billed against your credit balance."

Source: [OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml).

### Other useful endpoints

- `PATCH /org/kb/nodes/{id}/raw` — edit text; saves a new version and reprocesses. `PUT` replaces title and text.
- `GET /org/kb/nodes/{id}/content?version=<n>` — read an earlier version.
- `POST /org/kb/folders` — create a folder. `PUT /org/kb/nodes/{id}/tags` — set tags.
- `GET /org/gaps` — unanswered questions and unsupported claims (needs Fetch product).
- `GET /org/credits/balance` → `credits_available`, `total_usage`, `credit_limit`.
- Header `X-Senso-Signals: off` — keep test searches out of the gap report (still costs credits).

Sources: [concepts](https://docs.senso.ai/docs/concepts), [OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml).

### Free tier, credits, limits, delays, gotchas

| Topic | Finding | Source |
|---|---|---|
| Credits | Hackathon orgs are "pre-loaded" (amount not stated). Public signup gives "$100 in Senso credits". Processing a document, a search, a generated page and scheduled prompt runs cost credits. Per-operation prices: **not found**. | [FAQ doc](https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing), [essay](https://senso.ai/self-improving-roi), [concepts](https://docs.senso.ai/docs/concepts) |
| Out of credits | `402` with `code`: `insufficient_credits` or `spending_limit_reached`; retrying does not help | [errors](https://docs.senso.ai/docs/errors) |
| Rate limits | Only two documented: `POST /org/competitors/suggest` 5/hour; MCP 60 requests/minute per connection. No general REST limit is published | [limits](https://docs.senso.ai/docs/limits), [mcp-server](https://docs.senso.ai/docs/mcp-server) |
| Processing delay | "A short document is usually ready within a few seconds"; long files take longer. Not searchable before `complete` | [quickstart](https://docs.senso.ai/docs/quickstart) |
| Duplicate text | Identical text → `409`. A seed script that re-runs will fail unless it handles this | [quickstart](https://docs.senso.ai/docs/quickstart) |
| Markdown files | `.md` uploads are refused; send markdown as text through `POST /org/kb/raw` / `senso kb create-raw` | [limits](https://docs.senso.ai/docs/limits) |
| Two IDs | `/org/kb/upload` returns a content id; `/org/kb/nodes/{id}` needs the node id. A KB document id sent to `/org/content/{id}` is a `400` | [errors](https://docs.senso.ai/docs/errors), [blog-to-social](https://docs.senso.ai/docs/blog-to-social) |
| `content_ids` without `require_scoped_ids` | The CLI flag text says omitting it "allow[s] fallback to all content"; the recipe says "content_ids only biases the search and unapproved chunks can still come back" | [cli-reference](https://docs.senso.ai/docs/cli-reference), [openai-agents-sdk](https://docs.senso.ai/docs/openai-agents-sdk) |
| Gaps only on empty results | "Only a search that finds nothing at all is recorded." If a loosely related chunk comes back, Senso may answer with a guess and file no gap. Apply your own score threshold | [shared-context](https://docs.senso.ai/docs/shared-context) |
| Evals scope | `kb_accuracy` checks claims "about your brand"; general-knowledge claims are bucketed `general` and never checked. Write claims that name the retailer; setting the brand kit's `brand_name` may help (untested) | [OpenAPI](https://docs.senso.ai/specs/sdk-api.yaml) |
| Product gating | Gaps need the Fetch product; prompts, generation and publishing need the GEO product; otherwise `403` "Your organization doesn't have access to this product". Whether a "Knowledge Base for Agents" org has Fetch or evals is **unverified** | [concepts](https://docs.senso.ai/docs/concepts), [errors](https://docs.senso.ai/docs/errors) |
| 404s | Plain-text `404 page not found` = wrong path (check `/api/v1`); JSON 404 = ID not visible to this key. Two past winning teams reported the REST API "returned 404 on every endpoint" and fell back to the CLI | [errors](https://docs.senso.ai/docs/errors), [policyDiff](https://devpost.com/software/policydiff-asn4xf), [Receipts](https://devpost.com/software/receipts-oizgbn) |
| Keys | `tgr_` prefix; CLI keys expire after 7 days; `senso login` needs an admin; never put a key in browser code | [api-keys](https://docs.senso.ai/docs/api-keys) |
| Deleting | Deleting a folder deletes everything in it; a document still processing answers `409` | [concepts](https://docs.senso.ai/docs/concepts) |
| Searches are recorded | MCP and API searches are saved to the org's search history | [mcp-server](https://docs.senso.ai/docs/mcp-server) |

---

## 9. Past hackathons Senso sponsored

All are tokens& (formerly Creators Corner) events. Senso says 30+ since September 2024 ([essay](https://senso.ai/self-improving-roi)); only those with a findable Senso prize are listed. Devpost blocks direct fetching, so pages were read through a reader proxy.

| Event | Date / place | Senso track | Prize | Senso winners |
|---|---|---|---|---|
| [Self-Evolving Agents Hackathon](https://self-evolving-agents.devpost.com/) ([Luma](https://luma.com/swarmhack)) | 2026-07-24, DG717, San Francisco | "Best use of Senso.ai" | Devpost: "2000 in credits", 1 winner. Senso's recap: five teams awarded, $2,500 credits each | **Ratchet, DeUna, Immune, SwarmAds, Greplica** ([essay](https://senso.ai/self-improving-roi)) |
| [Harness Engineering Hack](https://harness-hack.devpost.com/) | 2026-06-12, San Francisco | "Best use of Senso.ai" | 2k credits, 1 winner | Not found |
| [Agentic Engineering Hack](https://agentic-engineering-hack.devpost.com/) | Date not found | "3k in credits awarded to the project that uses Senso's content generation APIs to publish grounded, citeable content to cited.md (or another public destination). Ingestion alone won't qualify — we're rewarding teams that close the loop from knowledge base to published, agent-discoverable content." | 3k credits, 1 winner | Unverified (see below) |
| [Deep Agents Hackathon – RSAC 2026](https://deep-agents-hackathon.devpost.com/) | Date not found, San Francisco | "Best use of Senso.ai" | 3k / 2k / 1k credits | Not found |
| [Multimodal Frontier Hackathon](https://multimodal-frontier-hackathon.devpost.com/) | Date not found | "Best use of Senso.ai" | 3k / 2k / 1k credits | Not found |
| [Multiagents Hackathon](https://multiagents-hackathon.devpost.com/) | Date not found, London | "Best use of Senso.ai" | 2k credits | Not found |
| [NYC – AI Agents Hackathon](https://nyc-ai-agents-hackathon.devpost.com/) | Date not found, New York | "Best use of Senso.ai" | 2k credits | Not found |
| [The Future of Agents Hackathon](https://the-future-of-agents-hackathon.devpost.com/) | Date not found, San Francisco | "Best Use of Senso.ai" | 2k credits | Not found (winners are badged on Devpost, but none of the 17 badged pages names the Senso prize) |
| [Voice & Video AI Agents Hackathon](https://voice-video-ai-agents.devpost.com/) | Date not found | "Best use of Senso.ai" | $500 credits. Judges included Tom Nelson and Saroop Bharwani (Senso) | Not found |

### Winners with verified details (July 2026, from Senso's own recap)

Source for all five: [Self-Evolving Agents, Self-Improving ROI](https://senso.ai/self-improving-roi). Devpost / GitHub links for these projects: **not found**.

- **Ratchet** (Aayush Dixit) — coding agent that saves a bug fix only after an independent test proves it. With memory, AI calls per fix fell from 4.25 to 1 over five rounds; without, rose from 5 to 8. First encounter $1.65 and 3.2 s; afterwards $0.03 and 0.8 s. *Senso use:* each proven fix stored with its history (who found it, who verified it, reuse count).
- **DeUna** (Axel Cuevas, Anthony Lee, Andrea Urbina) — digital twin of a small-business owner selling over WhatsApp. When no policy exists (a trade-in with a smashed screen), the agent sends the owner three buttons; one tap writes a permanent rule. *Senso use:* "Senso holds everything the agent is allowed to say — every price, policy, and promise ... Even the price floor lives there, so nobody can haggle the agent below it." **This is the closest precedent to a shopping agent.**
- **Immune** (Subhan Poudel) — turns prompt-injection traces into guard rules that must block the attack, eight mutations, and leave twelve legitimate tasks untouched. *Senso use:* surviving rules stored as versioned security records.
- **SwarmAds** (team not named) — agents generate ad campaigns, test them in a simulated market, promote winners into reusable memory.
- **Greplica** (team not named) — ingested a repository into Senso and benchmarked four context strategies with context as the only variable.

### Agentic Engineering Hack: winning projects that used Senso (which one took the Senso prize is unverified)

These carry Devpost "Winner" badges, but the pages do not say which prize.

- **[PolicyGuard](https://devpost.com/software/policyguard)** ([GitHub](https://github.com/Asyboi/agentic-hack.git)) — "the compliance API for AI agents acting on the open web". Before any fetch, the planned action is checked against live policy text. Pre-ingested ToS documents with stable content IDs; "Senso search context returns ranked chunks scoped to the right document"; LLM proposes a verdict, a rule engine has the final say; ClickHouse logs the row; "Senso engine publish writes a citeable to cited.md so the next agent that asks the same question can cite our prior answer instead of paying again." Their lesson: "Publish beats ingest. Senso's prize criterion was the right one to chase." They also built demo-mode fixtures with the same schema so the stage path could not fail.
- **[Wage.MD](https://devpost.com/software/wage-md)** — turns US DOL filings into an agent-queryable, cited salary index. "Senso publishes each company brief to a permanent, public cited.md URL."
- **[Receipts](https://devpost.com/software/receipts-oizgbn)** — tracks whether public figures keep promises. Senso "saves AI-agent-discoverable drafts to the knowledge base". "The REST API wasn't accessible from our network. Pivoted to the Senso CLI's `engine draft` command which worked instantly."
- **[policyDiff](https://devpost.com/software/policydiff-asn4xf)** — payer-policy change detection; publishes evidence briefs to Senso. "The Senso REST API returned 404 on every endpoint we tried. We had to install and use the Senso CLI instead."

Other badged winners at other events that list Senso in their stack (won different prizes): [CareSync](https://devpost.com/software/caresync-2vmy3e), [PatchEd](https://devpost.com/software/patched), [BIAZ Finance](https://devpost.com/software/biaz-finance), [Euclid](https://devpost.com/software/euclid-k0fln8).

### Patterns in what wins

1. **Senso is the gate, not a document dump.** Winners used it as the single source of what the agent is allowed to say or do (DeUna's price floor, PolicyGuard's pre-action check). The Agentic Engineering track said outright that ingestion alone does not qualify.
2. **Verify before storing or acting.** Ratchet (independent test), Immune (gauntlet), PolicyGuard (rule engine has the final say). Senso's summary: "Capture an outcome. Verify it. Store it. Reuse it."
3. **A human answers once and the answer becomes permanent context.** DeUna's one-tap rule is the pattern Senso quoted most warmly.
4. **A number.** Every write-up leads with a measured before/after ($1.65 → $0.03; 10 of 10 blocked). Senso asks of each project "what it decided to keep, and what it stopped paying for."
5. **Lineage is visible.** Who found it, who verified it, which version, how often reused.
6. **Solo and small teams win.** Two of the five July winners were solo builders.
7. **Reliability under demo conditions.** Winning teams designed fallbacks (demo mode, CLI fallback) on day one.

---

## 10. Fit with "shopping agent with an audit trail"

Time costs assume one person on the Senso integration in a 5.5-hour build. They are estimates.

**Idea 1 — Grounded retrieval with a logged trail (shallow; 30–45 min). Qualifies.**
Seed one folder per fictional retailer via `POST /org/kb/raw`: catalog, return policy, shipping, warranty, promo terms, one doc per topic so `content_id` is meaningful. Agent tool calls `/org/search/context`; every decision row stores the audit fields from section 8. Seed first thing at 11:00 so processing is done before anything depends on it; make the seed script tolerate `409`.

**Idea 2 — Evidence ledger, scoped "delegation grant", and refusal (medium; +45–60 min). Competitive.**
Implement Senso's recipe: app-side ledger, typed `cited_content_ids`, reject fabricated or empty citations. Use `content_ids` + `require_scoped_ids: true` so the agent can only read the retailers the shopper approved (maps to "Identify: what did the customer delegate?" in the CEO's essay). Add a score threshold: below it the agent says what is missing and asks, as the FAQ's "Northstar" scenarios model (answerable / missing information / incomplete requirement). UI: a timeline where each step expands to the passage, document title, `version_id`, and vector/lexical ranks.

**Idea 3 — Pre-purchase claim gate with evals (medium-deep; +45–60 min). Likely differentiator.**
Before the action, the agent writes its justification ("Border 40 costs $50/month, includes US data, $10 activation fee"). Send it to `POST /org/evals/text` with `wait: true`. Show a per-claim table: `verified` with the KB `quote`, `conflict`, `unsupported`. Any `conflict` blocks checkout; `unsupported` asks the shopper. Store `eval_run_id` in the trail. Test in the first hour: latency can approach a minute, and confirm that claims are treated as `about_brand` and that the org is not product-gated.

**Idea 4 — Versioned policy change, live (small add-on to 1–3; +20–30 min). Strong demo beat.**
On stage, edit the return policy (30 → 14 days) with `PATCH /org/kb/nodes/{id}/raw`, wait for `complete`, rerun the same request. The agent's decision changes and the trail shows a new `version_id`; the earlier receipt still points at the old version, readable with `?version=`. This demonstrates "verified, attributable, versioned, and reversible" in about 30 seconds. Risk: reprocessing time on stage; rehearse and have a recorded fallback.

**Idea 5 — Shopper decisions become approved context (deep; +60–75 min).**
When context is missing and the shopper answers ("never substitute brands", "max $60/month"), write it to a `shopper-preferences` folder with tags `status:approved`, `owner:<shopper>`, `decided:<date>` and retrieve it on the next run, so the second purchase needs fewer questions. Report a number: clarifying questions or model calls on run 1 versus run 2. This is the July winners' pattern applied to shopping. Follow Senso's rule: save what the person approved, not what the model generated. If Fetch is enabled, also show `GET /org/gaps` as "what the agent could not answer".

**Stretch — x402-paid context or a published receipt (+90 min or more; not recommended).** On-thesis for the CEO but off-brief, and publishing likely needs the GEO product.

**Suggested plan:** Ideas 1 + 2 + 4 by lunch, Idea 3 after lunch, Idea 5 only if ahead. Mark every action as real, simulated or test mode, as the FAQ requires.

**Cross-sponsor note:** logging the Senso audit fields as event properties to Amplitude or a SingleStore table lets one trail serve two tracks.

---

## 11. Risks and open questions for the booth

**Risks**

- Processing delay and `409` on duplicate text break naive seed scripts and live demos.
- Product gating: gaps (Fetch) and possibly evals may `403` on hackathon orgs. Unverified; test at 11:05.
- Evals: up to about a minute with `wait: true`; only "brand" claims are checked.
- A loosely relevant chunk can produce a confident wrong answer with no gap filed; do not rely on Senso alone to detect missing context.
- Credits: amount pre-loaded and per-call cost are unknown; check `GET /org/credits/balance` after seeding.
- Past teams hit REST 404s (probably wrong base path) and fell back to the CLI; keep the CLI installed as a fallback.
- Node 22.12+ required for the CLI.
- Prize is credits on a platform that was acquired last month; long-term value depends on Zeta's plans.
- No Senso name on the judge list; main-stage judges may not weigh Senso depth, so the trail must be legible to a non-Senso judge too.

**Questions for Andy / the Senso booth**

1. Who judges the Senso track, and when: at the table during the build, from the submission video, or only among finalists?
2. Is Saroop or Tom here today?
3. Does a "Knowledge Base for Agents" org have evals and the Fetch product (gaps) enabled? If not, can you enable them for us?
4. How many credits are pre-loaded, and what do a search, a document ingest and an eval cost?
5. Any REST rate limit we should know about for an agent making several searches per step?
6. For `kb_accuracy`, how does the judge decide a claim is "about your brand"? Do we need to set the brand kit to the fictional retailer?
7. Is `/org/search/context` or `/org/search` what you would rather see in a shopping agent?
8. Typical reprocessing time after `PATCH .../raw` on a short document? We want to change a policy live.
9. Would writing shopper-approved preferences back into the KB count for or against us ("save what a person approved")?
10. Is "10k in credits" 10,000 credits or $10,000? Is there a follow-on programme like the July one ($2,500 → $10,000 ROI)?
11. What made DeUna and Ratchet stand out to you, and what did losing Senso-track projects usually miss?

---

## 12. Sources

**Senso docs**
- https://docs.senso.ai/llms.txt
- https://docs.senso.ai/llms-full.txt
- https://docs.senso.ai/specs/sdk-api.yaml (OpenAPI, version 1.13.0)
- https://docs.senso.ai/docs/overview
- https://docs.senso.ai/docs/quickstart
- https://docs.senso.ai/docs/concepts
- https://docs.senso.ai/docs/why-senso
- https://docs.senso.ai/docs/mcp-server
- https://docs.senso.ai/docs/senso-cli
- https://docs.senso.ai/docs/cli-reference
- https://docs.senso.ai/docs/api-keys
- https://docs.senso.ai/docs/permissions
- https://docs.senso.ai/docs/errors
- https://docs.senso.ai/docs/limits
- https://docs.senso.ai/docs/agent-skills
- https://docs.senso.ai/docs/shared-context
- https://docs.senso.ai/docs/openai-agents-sdk
- https://docs.senso.ai/docs/x402
- https://docs.senso.ai/docs/use-cases/verification-loop
- https://docs.senso.ai/docs/blog-to-social

**Senso site**
- https://senso.ai and https://senso.ai/index.md
- https://www.senso.ai/llms.txt
- https://senso.ai/about
- https://senso.ai/your-next-customer-isnt-human
- https://senso.ai/self-improving-roi
- https://senso.ai/cited-md
- https://senso.ai/first-agentic-loop
- https://senso.ai/blog
- https://senso.ai/blog/fundmoreai-cited-before-the-first-sales-call-

**Event**
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack
- https://docs.google.com/document/d/17tFdnTaLQAFhe1_k0k1Ve_kLov9HHO7pyenw0X8Pv04/edit?usp=sharing (organizers' Senso FAQ, MCP setup, example MCP flow)

**Company and acquisition**
- https://www.ycombinator.com/companies/senso
- https://www.zetaglobal.com/news/zeta-global-to-acquire-senso-adding-ai-engineering-and-frontier-model-connectivity-to-its-intelligent-ai-infrastructure/
- https://ppc.land/zeta-global-to-buy-senso-the-startup-behind-its-ai-search-product/
- https://www.sec.gov/Archives/edgar/data/0001851003/000119312526453696/zeta-20260930.htm (via search summary only)
- https://www.vcbacked.co/company/sensoai (funding figure, unverified)
- https://www.cbinsights.com/company/sensoai/alternatives-competitors (funding figure, unverified)
- https://x.com/saroopbharwani/status/2081765280728985742 (via search result snippet)

**Past hackathons** (Devpost pages read through r.jina.ai because Devpost returns 403 to direct fetches)
- https://self-evolving-agents.devpost.com/ and https://luma.com/swarmhack
- https://harness-hack.devpost.com/ and https://luma.com/harnesshack
- https://agentic-engineering-hack.devpost.com/ and https://luma.com/shiptoprod
- https://deep-agents-hackathon.devpost.com/ and https://luma.com/deepagentshack
- https://multimodal-frontier-hackathon.devpost.com/
- https://multiagents-hackathon.devpost.com/
- https://nyc-ai-agents-hackathon.devpost.com/
- https://the-future-of-agents-hackathon.devpost.com/
- https://voice-video-ai-agents.devpost.com/ and https://luma.com/i8bow7sr
- https://devpost.com/software/policyguard
- https://devpost.com/software/wage-md
- https://devpost.com/software/receipts-oizgbn
- https://devpost.com/software/policydiff-asn4xf
- https://devpost.com/software/caresync-2vmy3e
- https://devpost.com/software/patched
- https://devpost.com/software/biaz-finance
- https://devpost.com/software/euclid-k0fln8
