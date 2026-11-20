# Airbyte — sponsor research (Personal Agents Hackathon, Nov 20 2026)

> Researched Nov 19 2026. Everything is cited inline. "Unverified" / "not found" means I looked and could not confirm it.
> Airbyte's agent product names changed three times in 12 months; read section 2 before talking to the booth.

## 1. TL;DR

- **What it is:** Airbyte is the open-source ELT company (700+ connectors) that now sells itself as "the context layer for AI agents". The page the organizers linked describes the **Airbyte context layer**: a Cloud-only feature that lets an agent, through the hosted **Airbyte MCP** (`https://mcp.airbyte.com/mcp`), (a) do read-only live reads from sources ("agent access") and (b) keyword/semantic/hybrid search over synced data ("search") ([docs](https://docs.airbyte.com/platform/context-layer)).
- **What the prize wants:** The track is unnamed on the event page; it only lists $1,000 / $500 / $250 Visa gift cards and the link "Airbyte Context docs" ([event page](https://tokensand.com/personalagenthack)). At the same organizer's earlier events the identical prize was called "Conquer with Context" ([Devpost, Mar 2026](https://deep-agents-hackathon.devpost.com/), [Devpost, Jun 2026](https://harness-hack.devpost.com/)). Inference: they want an agent whose decisions are visibly grounded in real data pulled through Airbyte.
- **Fastest qualifying path (about 30–45 min):** Airbyte Cloud 14-day trial (no card) → add one source (Shopify dev store, Stripe test key, Gmail or Google Sheets) → Context layer > Settings > turn on Agent access → connect the Airbyte MCP to the agent (OAuth in a browser, or `Client-Id` / `Client-Secret` headers for a headless app) → agent calls `get_external_access_skill_docs` then `execute_external_api_query` and you log each call as an audit-trail row.
- **The action half:** the context layer is strictly read-only. For "take a meaningful action" use the open-source `airbyte-agent-sdk` (Python ≥ 3.11), whose Shopify connector can create and complete draft orders and whose Stripe connector can create Checkout Sessions, Payment Intents and refunds ([Shopify connector README](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/shopify/README.md), [Stripe connector README](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/stripe/README.md)).
- **Biggest risks:** search needs a sync into a destination (backfill only for Snowflake/BigQuery) so it is slow to stand up; the older hosted "Agent Engine" docs now redirect to the docs homepage and `api.airbyte.ai` returned HTTP 530 when I probed it on Nov 19, so confirm at the booth which surface they want before building on hosted mode.

## 2. What they do

### Core business
Open-source data replication (ELT): move data from sources (SaaS APIs, databases, files) into warehouses, lakes and databases. The docs homepage now describes Airbyte as an "open source data replication platform and context layer for AI agents" ([docs home](https://docs.airbyte.com/)). Editions: self-managed Airbyte Core (free), Airbyte Cloud (Standard from $20/mo, Plus from $189/mo, Pro custom), Enterprise Flex (hybrid) ([pricing](https://airbyte.com/pricing)).

### Agent-facing product, as it exists on Nov 19 2026

| Piece | What it is | Source |
|---|---|---|
| **Context layer** | Umbrella for agent access + search. Cloud only, off by default, turned on by an org admin. Free, capped at 100,000 direct reads and searches per organization per month. | [context-layer](https://docs.airbyte.com/platform/context-layer) |
| **Agent access** | Read-only live reads from a source's API (`list`, `get`, and the connector's own `search` where offered) using credentials already stored in Airbyte; no connection or sync needed; the agent never sees the credentials. Also read-only SQL on BigQuery and Snowflake destinations (`SELECT`, `SHOW TABLES`). | [agent-access](https://docs.airbyte.com/platform/context-layer/agent-access.md) |
| **Search** | Index Airbyte builds from synced connection data, stored on Airbyte infrastructure. Keyword, semantic, hybrid (default). Source and destination must both be "agent connectors". Only syncs after enablement are indexed; older data needs a backfill, which supports Snowflake and BigQuery only. | [search](https://docs.airbyte.com/platform/context-layer/search.md) |
| **Airbyte MCP** | Hosted remote MCP server at `https://mcp.airbyte.com/mcp`. OAuth sign-in with the Airbyte Cloud account; agent acts with the user's permissions. Also builds/manages/troubleshoots pipelines. | [airbyte-mcp](https://docs.airbyte.com/platform/airbyte-mcp.md), [install](https://docs.airbyte.com/platform/airbyte-mcp/install.md) |
| **Agent connectors** | A connector capability flag. Each connector page shows "Connector type: Data replication / Agent". Agent connectors "support agent access and search through the context layer". | [connector capabilities](https://docs.airbyte.com/platform/move-data/sources-destinations-connectors.md) |
| **Airbyte Agent SDK** (`airbyte-agent-sdk`) | Open-source Python package: "50+ connectors with typed tools for third-party APIs. Run connectors locally with your own credentials"; the SDK calls each third-party API directly. Tool builders for PydanticAI, LangChain, OpenAI Agents, FastMCP. Supports writes on many connectors. | [README](https://github.com/airbytehq/airbyte-agent-sdk) |
| **PyAirbyte MCP** (`airbyte-mcp` from the `airbyte` PyPI package) | Local stdio MCP server, used for self-managed Core. No context layer on Core. | [Herrli blog, Nov 7 2026](https://airbyte.com/blog/connect-codex-to-airbyte-core-with-airbyte-mcp) |

### Naming timeline (why old blog posts will confuse you)
- **Dec 2025:** "agent connectors" launched as open-source Python SDKs ([newsletter index](https://docs.airbyte.com/llms.txt); summary via [search coverage](https://docs.airbyte.com/community/newsletter-archive/may-2026-three-ways-to-build-with-airbyte-agents)).
- **Feb 26 2026:** "Agent Engine" public beta: managed auth incl. OAuth, hosted agent connectors, and the "Context Store" ([newsletter](https://airbyte.com/community/newsletter/feb-2026-context-store)).
- **May 5 2026:** "Airbyte Agents" launch: Agent MCP, Agent SDK, Automations (research preview). Claimed "up to 80% fewer tokens and 40% fewer tool calls" versus vendor MCPs/APIs ([newsletter](https://docs.airbyte.com/community/newsletter-archive/may-2026-three-ways-to-build-with-airbyte-agents), [SDK blog](https://airbyte.com/blog/agent-sdk)).
- **Jun 17 / Jul 16 / Aug 18 2026:** team workspaces; agent writes to Salesforce and HubSpot; semantic search for Google Drive and Granola ([newsletter index](https://docs.airbyte.com/llms.txt)).
- **Oct 28 2026 (article date):** Airbyte MCP server announced, with the "new context layer" for Cloud and Enterprise Flex users: direct queries on Snowflake and BigQuery, data from 50+ source connectors ([destinationCRM](https://www.destinationcrm.com/Articles/ReadArticle.aspx?ArticleID=177251)).
- **Nov 19 2026 (my probe):** every `docs.airbyte.com/ai-agents/...` URL returns a 301 to the docs homepage; `app.airbyte.ai` returns 200; `api.airbyte.ai` returns HTTP 530. The SDK README now leads with local mode and says "Fast search across your data and context-layer queries aren't part of this SDK. They're available in Airbyte Cloud through the Airbyte MCP" ([README](https://github.com/airbytehq/airbyte-agent-sdk)). Whether the hosted Agent Engine has been retired is **unverified**; no announcement found.

### Positioning
Marketing line: "Your warehouse is your context" — AI tools search and query data already synced to the warehouse and fetch live records from 50+ sources; read-only; "every action is logged with the user, tool, and target" ([airbyte.com/context-layer](https://airbyte.com/context-layer)).

### Competitors (my characterization, not sourced from Airbyte)
- Agent tool/auth layers: Composio, Arcade, Nango, Pipedream, Merge. Airbyte's angle against them is the replicated, pre-indexed copy of the data plus a large connector catalog.
- ELT incumbents: Fivetran, Stitch, Meltano. Miro's case study describes moving off Stitch ([Miro story](https://airbyte.com/success-stories/miro)).
- At this event: Senso also uses the tagline "Context Layer" (see event brief). Senso is a curated knowledge base; Airbyte is live access to systems of record. They are complementary, and using both lets you say so explicitly.

## 3. Company facts

- **Founders:** Michel Tricot (CEO) and John Lafleur; founded 2020; San Francisco ([Contrary Research](https://research.contrary.com/company/airbyte), [Series B release](https://www.businesswire.com/news/home/20211217005648/en/Airbyte-Closes-150-Million-Series-B-Funding-Round-Led-by-Altimeter-Capital-and-Coatue-Management)).
- **Funding:** Series A $26M led by Benchmark, May 2021 ([Business Wire](https://www.businesswire.com/news/home/20210525005226/en/Open-Source-Data-Integration-Company-Closes-%2426-Million-Series-A-Funding-Round-Led-by-Benchmark)). Series B $150M, Dec 2021, led by Altimeter and Coatue with Thrive, Salesforce Ventures, Benchmark, Accel, SV Angel; $1.5B valuation; about $181M total ([Business Wire](https://www.businesswire.com/news/home/20211217005648/en/Airbyte-Closes-150-Million-Series-B-Funding-Round-Led-by-Altimeter-Capital-and-Coatue-Management)).
- **Later rounds:** one pre-IPO tracker claims a $200M Series C in June 2026 and $501M total; the same page misdates the Series B, and I found no primary source. Treat as **unverified** ([techstackipo](https://www.techstackipo.com/company/airbyte)).
- **Size:** about 140 employees per Contrary Research (updated Oct 2026); 51–200 band elsewhere ([Contrary](https://research.contrary.com/company/airbyte), [Simplify](https://simplify.jobs/c/Airbyte)).
- **Self-reported scale:** 25,000+ companies, 2M pipelines synced daily, "1 in 5 Fortune 500" ([airbyte.com/context-layer](https://airbyte.com/context-layer)).
- **Recent news:** Airbyte Core 2.1 (Apr 3 2026) and 2.2 (blog Aug 24 2026); Airbyte Agents launch (May 5 2026); MCP server + context layer (Oct 28 2026) (sources in section 2; [Herrli author page](https://airbyte.com/blog-authors/richelle-herrli)).

## 4. Customers & use cases

- **Logos on the MCP/agents page:** Carlsberg Group, Air Liquide, Wise, Miro, monday.com, Perplexity, Cloudinary, TUI Group, Bumble, among about 24 ([airbyte.com/agents](https://airbyte.com/agents)).
- **Case studies:** Perplexity (Mar 2024, one-person data team) ([story](https://airbyte.com/success-stories/perplexity)); Miro (Jul 6 2026, consolidating third-party ingestion into Snowflake) ([story](https://airbyte.com/success-stories/miro)). Index of stories: [airbyte.com/case-studies](https://airbyte.com/case-studies).
- **Agent-product early users quoted at the May 2026 launch:** Nate Chambers, CPO of ORCA Analytics; Franziska Ibscher, Head of Product at Drivepoint ([newsletter](https://docs.airbyte.com/community/newsletter-archive/may-2026-three-ways-to-build-with-airbyte-agents)).
- **Use cases Airbyte itself pushes for the context layer:** check the current status of a support ticket, pull recent CRM opportunities, count last week's orders in the warehouse ([agent-access](https://docs.airbyte.com/platform/context-layer/agent-access.md)). These are B2B/ops examples; a consumer shopping agent is off their beaten path, which is an opening.
- **Commerce-adjacent content:** search results list Airbyte pages titled "Shopify chargeback defense agent", "e-commerce order dispute agent", and a tutorial/blog on fighting Stripe disputes with Airbyte Agents (blog by Roi Fang, Jun 12 2026). When fetched, the first two URLs served generic marketing content, so their details are **unverified** ([search hit](https://airbyte.com/use-case/shopify-chargeback-defense-agent), [author page](https://airbyte.com/blog-authors/roi-fang)).

## 5. Why they are at this hackathon

Inferred, not stated by Airbyte:

- **They are re-launching the agent story under a new name.** The context layer + MCP is three weeks old in the press (Oct 28 2026) and the organizers linked that page, not the SDK. They need developers to try the Cloud flow and produce demos.
- **They are converting an ELT brand into an agent-infrastructure brand.** The event page still says "Open-source data integration for 700+ sources" while Luma says "The Context Layer for AI Agents" ([event page](https://tokensand.com/personalagenthack), [Luma](https://luma.com/personalagenthack)).
- **Repeat sponsor of this organizer.** Same prize ladder ($1,000 / $500 / $250 Visa cards) at the Deep Agents Hackathon (Mar 27 2026) and Harness Engineering Hack (Jun 12 2026); credits at the Autonomous Agents Hackathon (Feb 27 2026) (section 9). Luma lists tokens& and AWS Builder Loft as hosts of the current event ([Luma](https://luma.com/personalagenthack)).
- **Behaviour they want:** a Cloud signup, a connected source, agent access switched on, the MCP wired into a real agent, and a story where grounded data changed the agent's decision.

## 6. Prize track

- **Exact wording on the event page:** heading "Airbyte"; "Open-source data integration for 700+ sources."; "1st place: $1,000 Visa gift card 2nd place: $500 Visa gift card 3rd place: $250 Visa gift card"; link "Airbyte Context docs" → `https://docs.airbyte.com/platform/context-layer`. No track name and no criteria are published ([event page](https://tokensand.com/personalagenthack)).
- **Likely general rubric:** the same organizer's Devpost events used Idea 20%, Technical implementation 20%, Tool use 20%, Presentation (3-minute demo) 20%, Autonomy 20% ([Devpost](https://deep-agents-hackathon.devpost.com/)). Not confirmed for this event.

### Richelle Herrli — judge and speaker
- Senior Developer Advocate at Airbyte ([author page](https://airbyte.com/blog-authors/richelle-herrli), [Luma](https://luma.com/personalagenthack)).
- LinkedIn: based in San Jose, CA; University of Massachusetts Amherst 2011–2015; English and German; recommendations describe "an exceptional technical project manager and developer advocate" and experience running scrum teams and writing technical style guides. A post from about four months ago says she had just finished her first month at Airbyte, so she joined around mid-2026 ([LinkedIn](https://www.linkedin.com/in/richelleherrli/)).
- Prior employer: a contact-data site lists Clover (Engineering Manager, developer advocacy/reporting/tooling). **Unverified** ([cience](https://www.cience.com/profile/richelle-herrli/59931279)).
- Her Airbyte posts (all 2026): six new community connectors (Aug 14); Airbyte 2.2 (Aug 24); Linear/Gong/YouTube certified (Sep 16); draft mode for sources and destinations (Sep 29); sync logs for Enterprise Flex (Oct 14); "Connect Codex to Self-Managed Airbyte Core with the Airbyte MCP" (Nov 7); MySQL, Gmail, Ashby, Pipedrive, Reddit Ads, Zoom certified (Nov 10) ([author page](https://airbyte.com/blog-authors/richelle-herrli)).
- LinkedIn activity: promoted the Airbyte MCP and context layer; built a Claude skill called "Dispute Fighter" that uses Airbyte Agents to gather documentation for Stripe disputes; ran a hands-on workshop at Data Outpost on Nov 4 about building trustworthy AI agents with Airbyte and MotherDuck ([LinkedIn](https://www.linkedin.com/in/richelleherrli/)).
- **Hackathon posts:** not found. No evidence she judged the earlier 2026 events.
- **What this implies:** she writes careful, security-conscious tutorials (env files with `chmod 600`, read-only vs write MCP registrations, approval gates on writes, never printing secrets) ([Nov 7 blog](https://airbyte.com/blog/connect-codex-to-airbyte-core-with-airbyte-mcp)). A demo that shows least-privilege access, read-only context, human approval before the purchase, and a commerce/Stripe angle maps onto things she has personally built and written about.

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **Context layer agent access through the Airbyte MCP, called by your agent at runtime.** This is the linked doc. Tools: `get_external_access_skill_docs`, `execute_external_api_query` (`list` / `get` / `search`), `execute_external_sql_query` ([tools](https://docs.airbyte.com/platform/airbyte-mcp/tools.md)).
2. **Context layer search** (`execute_external_search_query`, hybrid/semantic over synced data, plus `get_search_index_status`). Most differentiated feature and the least likely to be attempted by other teams, but the setup cost is real (section 11) ([tools](https://docs.airbyte.com/platform/airbyte-mcp/tools.md#search)).
3. **Multiple sources joined in one decision** (for example Shopify catalog + Gmail receipts + Google Sheets budget). Past winners all pulled from real systems; Deep Sentinel used three connectors (section 9).
4. **`airbyte-agent-sdk` connectors for the write/action step** (Shopify draft order, Stripe Checkout Session, Gmail send). Open source, actively released (v0.1.403 on Nov 16 2026) ([PyPI](https://pypi.org/project/airbyte-agent-sdk/), [repo](https://github.com/airbytehq/airbyte-agent-sdk)).
5. **Governance features shown on screen:** read-only mode, `X-MCP-Allow-Destructive-Tools: 0`, `X-Airbyte-Application-Name`, per-source agent access toggles ([advanced config](https://docs.airbyte.com/platform/airbyte-mcp/advanced-configuration.md), [manage access](https://docs.airbyte.com/platform/context-layer/manage-access.md)).
6. **PyAirbyte / classic sync** as supporting plumbing only. It has won before (FDA SafetyNet, section 9) but it is not what the linked page is about.

## 8. Quickstart

### 8.1 Sign up
- Airbyte Cloud: `https://cloud.airbyte.com/signup`. 14-day trial, 400 credits (about 100 GB of syncs), no credit card ([pricing](https://airbyte.com/pricing), [docs home](https://docs.airbyte.com/)).
- Plan fine print conflicts across pages: docs say "The context layer is free, with up to 100,000 direct reads and searches per organization per month" ([docs](https://docs.airbyte.com/platform/context-layer)); the product page says it is included on Cloud Standard, Plus, Pro and Enterprise Flex, not Core, and that the 14-day trial covers the context layer and semantic search ([product page](https://airbyte.com/context-layer)); the MCP page says "real-time queries and the context layer require a paid plan" ([airbyte.com/agents](https://airbyte.com/agents)). Working assumption: a fresh trial org is enough. Confirm at the booth.

### 8.2 Turn on the context layer (UI)
From [manage access](https://docs.airbyte.com/platform/context-layer/manage-access.md):
1. Add a source in the workspace (credentials are entered once in Airbyte; OAuth where the connector supports it).
2. As org admin: **Context layer > Settings > Agent access** on. This covers all supported sources; destinations are not enabled automatically.
3. **Context layer > Agent access**: confirm the per-source toggle (on by default for new supported sources).
4. Optional search: **Context layer > Settings > Semantic search** on and accept the terms; then toggle per connection under **Context layer > Semantic search**.

### 8.3 Connect the Airbyte MCP
Interactive (Claude Code), verbatim from [install](https://docs.airbyte.com/platform/airbyte-mcp/install.md):

```bash
claude mcp add --transport http airbyte https://mcp.airbyte.com/mcp
```
then authenticate with `/mcp`. Cursor / generic clients:

```json
{
  "mcpServers": {
    "airbyte": {
      "url": "https://mcp.airbyte.com/mcp"
    }
  }
}
```

Headless (what a shipped agent needs). Create an Airbyte application, copy its client ID and secret, and send them on every request; the MCP exchanges them for a token per request, so no token endpoint is needed. Do not put them in the client's OAuth fields ([install](https://docs.airbyte.com/platform/airbyte-mcp/install.md)):

```json
{
  "mcpServers": {
    "airbyte": {
      "url": "https://mcp.airbyte.com/mcp",
      "headers": {
        "Client-Id": "<CLIENT_ID>",
        "Client-Secret": "<CLIENT_SECRET>"
      }
    }
  }
}
```
Optional headers: `X-Airbyte-Workspace-Id`, `X-Airbyte-Organization-Id`, `X-MCP-Readonly-Mode: 1` (or URL `?read-only=true`), `X-MCP-Allow-Destructive-Tools: 0`, `X-MCP-Allow-Pipeline-Changes: 0`, `X-MCP-Exclude-Tools`, `X-Airbyte-Application-Name` ([advanced config](https://docs.airbyte.com/platform/airbyte-mcp/advanced-configuration.md)).

Calling it from your own Python agent. **This snippet is my adaptation using the generic `mcp` Python client, not Airbyte's code.** Airbyte's tools page does not document the argument names for `execute_external_api_query`, so list the tools and read the schema first:

```python
import asyncio, os
from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client

HEADERS = {
    "Client-Id": os.environ["AIRBYTE_CLIENT_ID"],
    "Client-Secret": os.environ["AIRBYTE_CLIENT_SECRET"],
    "X-MCP-Readonly-Mode": "1",
    "X-Airbyte-Application-Name": "shopping-agent",
}

async def main():
    async with streamablehttp_client("https://mcp.airbyte.com/mcp", headers=HEADERS) as (r, w, _):
        async with ClientSession(r, w) as s:
            await s.initialize()
            tools = await s.list_tools()
            for t in tools.tools:
                if t.name.startswith(("execute_external", "get_external")):
                    print(t.name, t.inputSchema)   # read the real parameter names here

asyncio.run(main())
```

Tool names that matter ([tools](https://docs.airbyte.com/platform/airbyte-mcp/tools.md)):
- `list_connectors`, `describe_connector` (includes direct read instructions)
- `get_external_access_skill_docs` — entities, actions, parameters for a connector; agents read this first
- `execute_external_api_query` — `list`, `get`, `search` against the source API
- `execute_external_sql_query` — read-only SQL on Snowflake/BigQuery
- `execute_external_search_query` — documented params: `prompt`, `search_type` (`keyword` | `semantic` | `hybrid`, default `hybrid`), `limit` (1–100, default 10), `streams` (each with `stream_name`, optional `namespace`, `source_id`, `fields`), `lookback_seconds`, `min_similarity`, `max_similarity_diff`, `max_context_chars`, `destination_id`
- `get_search_index_status`, `request_search_index_backfill`

### 8.4 Agent SDK (local mode, includes writes)
From the [README](https://github.com/airbytehq/airbyte-agent-sdk), verbatim:

```bash
uv pip install airbyte-agent-sdk
```

```python
import os

from pydantic_ai import Agent
from airbyte_agent_sdk import build_connector_tools
from airbyte_agent_sdk.connectors.stripe import StripeConnector, StripeAuthConfig

stripe = StripeConnector(
    auth_config=StripeAuthConfig(api_key=os.environ["STRIPE_API_KEY"])
)
tools = build_connector_tools(stripe, framework="pydantic_ai")

agent = Agent("openai:gpt-4o", tools=tools.as_list())
```

The builder gives the model three tools — `inspect_connector`, `read_skill_docs`, `execute` — and the names are fixed, so **one connector per agent** with this builder. For several connectors use `@<Connector>.agent_tool(...)` with your own function names ([README](https://github.com/airbytehq/airbyte-agent-sdk)).

Shopify, local credentials, verbatim from the connector's [AUTH.md](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/shopify/AUTH.md):

```python
from airbyte_agent_sdk.connectors.shopify import ShopifyConnector
from airbyte_agent_sdk.connectors.shopify.models import ShopifyAccessTokenAuthenticationAuthConfig

connector = ShopifyConnector(
    auth_config=ShopifyAccessTokenAuthenticationAuthConfig(
        api_key="<Your Shopify Admin API access token>"
    ),
    shop="<Your Shopify store name (e.g., 'my-store' from my-store.myshopify.com)>"
)
```

Direct call shape used in the docs: `await connector.execute(entity, action, params or {})` ([AUTH.md](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/shopify/AUTH.md)). Exact entity/action strings and params are in each connector's `REFERENCE.md`; read them rather than guessing.

Hosted mode, as still documented in the connector AUTH files: `connect("shopify", workspace_name="<your_workspace_name>")` reading `AIRBYTE_CLIENT_ID` / `AIRBYTE_CLIENT_SECRET`, with connectors created by `airbyte-agent login` + `airbyte-agent connectors create` or `POST https://api.airbyte.ai/api/v1/integrations/connectors`. The quickstart it links to now redirects to the docs homepage and the API host returned HTTP 530 on Nov 19. **Do not plan around hosted mode without booth confirmation.**

Skills for coding agents ([README](https://github.com/airbytehq/airbyte-agent-sdk)):
```bash
npx skills add airbytehq/airbyte-agent-sdk
```

### 8.5 Connectors relevant to shopping

| Connector | Agent access in Cloud (connector page says "Data replication, Agent") | In `airbyte-agent-sdk` | Useful entities / writes |
|---|---|---|---|
| Shopify | Yes ([docs](https://docs.airbyte.com/integrations/sources/shopify)) | Yes | Products, variants, inventory levels, price rules, discount codes, orders, abandoned checkouts. Writes: create/complete draft order, create discount code, create customer. Cannot process a refund or edit order lines after creation ([README](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/shopify/README.md)) |
| Stripe | Yes ([docs](https://docs.airbyte.com/integrations/sources/stripe)) | Yes | Charges, refunds (create), products, payment intents (create/confirm/cancel), checkout sessions (create), disputes. Key starts `sk_test_` or `sk_live_` ([README](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/stripe/README.md)) |
| Gmail | Yes; Airbyte-certified as of Nov 10 2026 ([docs](https://docs.airbyte.com/integrations/sources/gmail), [blog list](https://airbyte.com/blog-authors/richelle-herrli)) | Yes | Messages list/get/search, threads, send, drafts, labels. OAuth only (refresh token); no attachments or forwarding ([README](https://github.com/airbytehq/airbyte-agent-sdk/blob/main/connectors/gmail/README.md)) |
| Google Sheets | Yes ([docs](https://docs.airbyte.com/integrations/sources/google-sheets)) | No (not in the repo's connector list) | Budget, wishlist, household inventory |
| Google Drive | Yes ([docs](https://docs.airbyte.com/integrations/sources/google-drive)) | Yes | Receipts, manuals, warranty PDFs |
| WooCommerce | Yes ([docs](https://docs.airbyte.com/integrations/sources/woocommerce)) | Yes | Alternative store backend |
| Amazon Seller Partner / Amazon Ads | Yes ([docs](https://docs.airbyte.com/integrations/sources/amazon-seller-partner)) | Yes | Seller-side data only. There is no consumer Amazon order-history connector |
| Notion, Slack | Yes | Yes | Shared lists, approval messages |
| Exa | not checked | Yes | Web search for product research |

SDK connector list read from the repo on Nov 19 2026 ([connectors dir](https://github.com/airbytehq/airbyte-agent-sdk/tree/main/connectors)).

### 8.6 Gotchas
- Context layer is Cloud only; nothing on self-managed Core ([docs](https://docs.airbyte.com/platform/context-layer)).
- Direct reads are read-only and "cannot create, update, or delete records" ([agent-access](https://docs.airbyte.com/platform/context-layer/agent-access.md)).
- Direct reads count against the source's own API rate limits ([docs](https://docs.airbyte.com/platform/context-layer)).
- Direct reads use the credentials saved on the connector, which may see more than the signed-in user should; use a narrow key ([manage access](https://docs.airbyte.com/platform/context-layer/manage-access.md)).
- Shopify OAuth works only in Airbyte Cloud; elsewhere use an Admin API access token from a custom app ([Shopify docs](https://docs.airbyte.com/integrations/sources/shopify)).
- Search: only data synced after enablement is indexed; backfill is Snowflake/BigQuery only; turning search off deletes the index ([search](https://docs.airbyte.com/platform/context-layer/search.md)).
- Headless bearer tokens expire after 15 minutes; prefer the `Client-Id` / `Client-Secret` headers ([install](https://docs.airbyte.com/platform/airbyte-mcp/install.md)).
- SDK needs Python ≥ 3.11 and ships almost daily, so pin the version ([PyPI](https://pypi.org/project/airbyte-agent-sdk/)).
- In local SDK mode, `context_store_search` and `context_store_sql_query` actions raise `NotImplementedError` ([README](https://github.com/airbytehq/airbyte-agent-sdk)).
- SDK tool output over 100 KB is turned into a retry signal; ask for small pages ([README](https://github.com/airbytehq/airbyte-agent-sdk)).
- A past winner hit a bug in the GitHub agent connector mid-event (section 9). Test the exact entity/action you need before the clock starts.

## 9. Past hackathons

### 9.1 Same organizer series, 2026 (verified from Devpost project pages)

**Autonomous Agents Hackathon — Feb 27 2026.** Prize: "Best Use of Airbyte", 2 winners, $1,000 / $500 in credits ([Devpost](https://autonomous-agents-hackathon.devpost.com/)).
- **RivalMap** (also Top 3 Overall): takes a company URL and produces a competitive-intelligence report. Airbyte use: pulls pitch decks, briefs and prior analyses from a connected Google Drive through the Airbyte Agent Engine (OAuth + pull, env vars `AIRBYTE_CLIENT_ID`, `AIRBYTE_CLIENT_SECRET`, `AIRBYTE_CONNECTOR_ID`) ([Devpost](https://devpost.com/software/rivalmap), [repo](https://github.com/AlfredSjoqvist/autonomous-agents)).
- **backoffice.ai** (also Best Use of Neo4j): AI back-office workforce for founders. Airbyte described as the "universal connector layer" for syncing employee data to Notion, Salesforce, Stripe, Jira, and marked "[In Progress]" in the write-up; built-with tag "airbyte (pyairbyte + agent connectors)" ([Devpost](https://devpost.com/software/backoffice-ai), [repo](https://github.com/yajatns/Feb27Hackathon)).

**Deep Agents Hackathon — Mar 27 2026, AWS Builder Loft.** Prize: "Airbyte: Conquer with Context", 3 winners, $1,000 / $500 / $250 Visa gift cards ([Devpost](https://deep-agents-hackathon.devpost.com/)). Placement order among the three is not shown.
- **Sift**: monitors product feedback, scores severity, phones the founder when critical. Airbyte use: agent connectors' typed Python SDK pulling real GitHub issues, PRs and commits ([Devpost](https://devpost.com/software/sift-yowran), [repo](https://github.com/UJameel/Sift)).
- **Deep Sentinel**: correlates GitHub PRs, Slack discussion and codebase architecture to surface risks scanners miss. Airbyte use: three connectors (GitHub, Slack, Jira) via the agent SDK. They hit and fixed a GraphQL template bug in the GitHub connector during the event ([Devpost](https://devpost.com/software/deep-sentinel), [repo](https://github.com/ElijahUmana/deep-sentinel)).
- **Signal**: deep agent for product teams producing decision briefs. Airbyte use: hosted agent connectors for Linear, GitHub, Asana, Sentry, Typeform so the app never stores raw third-party credentials ([Devpost](https://devpost.com/software/signal-0gs579), [repo](https://github.com/akshsgaur/signal_machine)).

**Harness Engineering Hack — Jun 12 2026.** Prize: "Conquer with Context: Best Use of Airbyte's Agent Engine", 3 winners, $1,000 / $500 / $250 Visa gift cards ([Devpost](https://harness-hack.devpost.com/)).
- **FDA SafetyNet** (also Top 2 Overall): cross-references FDA drug recalls with patient records and sends alerts. Airbyte use: PyAirbyte declarative source pulling the openFDA enforcement API incrementally into ClickHouse ([Devpost](https://devpost.com/software/silvershield-ofgwi2), [repo](https://github.com/mansiguptacs/FDASafetyNet)).
- **Hestia**: event-ops agent harness with sub-agents for venue, catering, vendors. Airbyte use: orchestrator "pulls shared context from Notion through Airbyte" ([Devpost](https://devpost.com/software/hestia-lbjg7v), [repo](https://github.com/alek-karp/hestia-ai)).
- **SayHello!**: GTM harness that researches the story behind each lead. Airbyte use: "Every grounded story syncs into Airbyte's Context Store, so the agent can recall it later" ([Devpost](https://devpost.com/software/sayhello-y3lszj), [repo](https://github.com/Jshengdev/sayhello)).

Airbyte staff at these events: a LinkedIn result says Airbyte engineer Pedro S. Lopez spoke and judged at the Autonomous Agents Hackathon ([LinkedIn](https://www.linkedin.com/in/pedroslopez/)). **Unverified** beyond the search snippet.

### 9.2 Airbyte's own online hackathons (older, ELT era)
Airbyte + MotherDuck ($10,000 pool, Jan 2025), Airbyte + MindsDB ($5,000 pool), PyAirbyte tutorial hackathon (June 2024), Airbyte x Hashnode, Hacktoberfest connector hackathons, Airbyte + dbt ([MotherDuck winners](https://airbyte.com/blog/and-the-winner-of-the-airbyte-motherduck-hackathon-is), [MindsDB winners](https://airbyte.com/blog/winner-of-the-airbyte-mindsdb-hackathon), [PyAirbyte winners](https://airbyte.com/blog/announcing-pyairbyte-hackathon-winners), [Hashnode winners](https://townhall.hashnode.com/airbyte-x-hashnode-winners)). I did not open the individual winner posts; winner names for these are **not researched**. They are pipeline/tutorial contests and are weak predictors for this event.

No Airbyte-sponsored in-person hackathon between July and October 2026 was found.

### 9.3 Patterns in what wins
- **Real data from a real system, shown live.** Every winner pulled from an actual account or public API. None relied on mock JSON.
- **Airbyte as the context/ingest step, not the centerpiece.** Winners used it for one clear job and were strong products overall; 5 of 8 also won another sponsor prize or an overall prize.
- **The bar for depth has been moderate.** A project with Airbyte marked "In Progress" won in February, and a PyAirbyte ingestion pipeline won a prize named for the Agent Engine in June. A team that uses the actual context layer features well would stand out.
- **Credential hygiene gets called out.** Signal's write-up leads with not storing third-party credentials.
- **Multi-connector joins and explicit "why this matters" framing** recur (Deep Sentinel, SayHello!).
- **Nobody has won with the new MCP-based context layer yet**, because it post-dates all three events. First credible use of agent access + search is open ground.

## 10. Fit with "shopping agent with an audit trail"

Mapping to the brief: Airbyte covers "reliable context" (live reads from systems of record) and contributes to "show the steps" (each read is a discrete, loggable tool call with connector, entity, action and record IDs). It does not do the purchase; that is the SDK or your own code.

| # | Idea | Airbyte surface | Time in a 5.5h build |
|---|---|---|---|
| 1 | **Grounded catalog reads.** Agent reads live price, stock and active discount codes from a Shopify dev store before recommending. Each `execute_external_api_query` call becomes an audit row: timestamp, connector, entity, action, filters, record IDs returned, "live as of request". | Cloud trial + agent access + Airbyte MCP (headless headers) | 30–45 min if the store and token exist beforehand |
| 2 | **Read with the context layer, act with the SDK.** After idea 1, create a Shopify draft order (or Stripe Checkout Session in test mode) via `airbyte-agent-sdk`, gated by a human approve button. Label it clearly as test mode. Read path and write path use separate credentials, read path in `X-MCP-Readonly-Mode`. | Above + `airbyte-agent-sdk` Shopify/Stripe | +45–60 min |
| 3 | **Personal context from the shopper's own systems.** Gmail receipts (what they already own, sizes, past prices paid), a Google Sheet budget/wishlist, plus the store catalog. The agent declines or downgrades a purchase when the budget sheet or a prior receipt contradicts it, and cites the exact message/row. | Agent access on Gmail + Google Sheets + Shopify | 1.5–2 h (Google OAuth consent is the slow part) |
| 4 | **Post-purchase guardian.** Inspired by Herrli's "Dispute Fighter": the agent watches Stripe charges and Gmail receipts, detects a price drop or duplicate charge, assembles the evidence pack, and files a test-mode refund with approval. | Agent access on Stripe + Gmail; SDK Stripe refunds create | 2–2.5 h |
| 5 | **Freshness-aware decisions with search.** Sync order history/receipts to BigQuery, enable semantic search, and have the agent use `execute_external_search_query` for "have I bought something like this before" and `execute_external_sql_query` for spend totals, then verify the winner with a live read. The audit trail shows indexed-as-of time versus live time for each fact. | Search + SQL + agent access | 2.5–3.5 h, high risk; only if BigQuery and a first sync are ready before kickoff |

Recommended scope: 1 + 2 as the baseline, 3 as the differentiator. Add 5 only if pre-event setup is permitted and working.

Audit-trail fields worth logging per Airbyte call: tool name, connector/source ID, entity, action, parameters, returned record IDs, row count, timestamp, whether the value was live or indexed, and which decision consumed it. Airbyte states its own side logs each action with user, tool and target ([product page](https://airbyte.com/context-layer)); whether that log is exportable is an open question.

## 11. Risks and open questions for the booth

1. **Which surface counts for the prize?** Context layer via Airbyte MCP only, or also `airbyte-agent-sdk` local mode and PyAirbyte? Past winners used all three.
2. **Is the hosted Agent Engine / `connect()` / `api.airbyte.ai` / `airbyte-agent` CLI still supported?** Docs redirect and the API host returned 530 on Nov 19.
3. **Does a fresh 14-day trial org get agent access and search?** Pages disagree (section 8.1). Is there a hackathon org or promo?
4. **Argument schema for `execute_external_api_query`** — not on the tools page. Is there an example payload?
5. **Search without a warehouse:** which destinations are agent connectors besides Snowflake and BigQuery? How long from first sync to searchable? Is there a pre-indexed demo workspace?
6. **Writes:** any write path through the Cloud MCP, or is the SDK the only way? (July 2026 newsletter mentions agent writes to Salesforce and HubSpot only.)
7. **Shopify/Gmail specifics:** does agent access work with a Shopify dev store token and a personal Gmail OAuth, and which entities does each expose for direct reads?
8. **Audit log:** can we read Airbyte's own log of MCP actions (user, tool, target) through an API to embed in our trail?
9. **Rate limits:** direct reads hit the source API quota; any caching?
10. **Judging:** does Richelle judge the Airbyte prize alone, and what does she weigh — depth of context layer use, number of sources, or product quality?
11. **Gmail as a data source for a demo:** direct reads use the stored credential's full scope. Use a throwaway inbox seeded with fake receipts, not a personal one.

## 12. Sources

Event
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack

Airbyte docs
- https://docs.airbyte.com/
- https://docs.airbyte.com/llms.txt
- https://docs.airbyte.com/platform/context-layer
- https://docs.airbyte.com/platform/context-layer/agent-access.md
- https://docs.airbyte.com/platform/context-layer/search.md
- https://docs.airbyte.com/platform/context-layer/manage-access.md
- https://docs.airbyte.com/platform/airbyte-mcp.md
- https://docs.airbyte.com/platform/airbyte-mcp/install.md
- https://docs.airbyte.com/platform/airbyte-mcp/tools.md
- https://docs.airbyte.com/platform/airbyte-mcp/advanced-configuration.md
- https://docs.airbyte.com/platform/move-data/sources-destinations-connectors.md
- https://docs.airbyte.com/integrations/sources/shopify (and `stripe`, `gmail`, `google-sheets`, `google-drive`, `woocommerce`, `amazon-seller-partner`, `notion`, `slack`, `postgres` under the same path)
- https://docs.airbyte.com/community/newsletter-archive/may-2026-three-ways-to-build-with-airbyte-agents
- https://airbyte.com/community/newsletter/feb-2026-context-store (via search summary)
- Probed on Nov 19 2026, all 301 to docs home: https://docs.airbyte.com/ai-agents/ , /ai-agents/interfaces/sdk/ , /ai-agents/interfaces/mcp/ , /ai-agents/get-started/developer-quickstart/

Airbyte site and blog
- https://airbyte.com/context-layer
- https://airbyte.com/agents
- https://airbyte.com/pricing
- https://airbyte.com/blog/agent-sdk
- https://airbyte.com/blog/connect-codex-to-airbyte-core-with-airbyte-mcp
- https://airbyte.com/blog-authors/richelle-herrli
- https://airbyte.com/blog-authors/roi-fang (search snippet only)
- https://airbyte.com/use-case/shopify-chargeback-defense-agent (served generic content)
- https://airbyte.com/tutorials/fighting-stripe-disputes-with-airbyte-agents (served tutorial index)
- https://airbyte.com/success-stories/perplexity , https://airbyte.com/success-stories/miro , https://airbyte.com/case-studies (search summaries)
- https://airbyte.com/blog/and-the-winner-of-the-airbyte-motherduck-hackathon-is , https://airbyte.com/blog/winner-of-the-airbyte-mindsdb-hackathon , https://airbyte.com/blog/announcing-pyairbyte-hackathon-winners , https://townhall.hashnode.com/airbyte-x-hashnode-winners (titles only)

Code
- https://github.com/airbytehq/airbyte-agent-sdk (README, `connectors/` listing, `connectors/shopify/README.md` and `AUTH.md`, `connectors/gmail/README.md` and `AUTH.md`, `connectors/stripe/README.md` and `AUTH.md`, commit log)
- https://pypi.org/project/airbyte-agent-sdk/

Past hackathons
- https://autonomous-agents-hackathon.devpost.com/ , https://deep-agents-hackathon.devpost.com/ , https://harness-hack.devpost.com/ (and each `/project-gallery`)
- https://devpost.com/software/rivalmap , https://devpost.com/software/backoffice-ai
- https://devpost.com/software/sift-yowran , https://devpost.com/software/deep-sentinel , https://devpost.com/software/signal-0gs579
- https://devpost.com/software/silvershield-ofgwi2 , https://devpost.com/software/hestia-lbjg7v , https://devpost.com/software/sayhello-y3lszj
- https://github.com/AlfredSjoqvist/autonomous-agents , https://github.com/UJameel/Sift , https://github.com/ElijahUmana/deep-sentinel , https://github.com/akshsgaur/signal_machine , https://github.com/mansiguptacs/FDASafetyNet , https://github.com/alek-karp/hestia-ai , https://github.com/Jshengdev/sayhello , https://github.com/yajatns/Feb27Hackathon

People and company
- https://www.linkedin.com/in/richelleherrli/
- https://www.cience.com/profile/richelle-herrli/59931279 (search snippet, unverified)
- https://www.linkedin.com/in/pedroslopez/ (search snippet)
- https://www.businesswire.com/news/home/20211217005648/en/Airbyte-Closes-150-Million-Series-B-Funding-Round-Led-by-Altimeter-Capital-and-Coatue-Management
- https://www.businesswire.com/news/home/20210525005226/en/Open-Source-Data-Integration-Company-Closes-%2426-Million-Series-A-Funding-Round-Led-by-Benchmark
- https://research.contrary.com/company/airbyte , https://simplify.jobs/c/Airbyte , https://www.techstackipo.com/company/airbyte (search summaries)
- https://www.destinationcrm.com/Articles/ReadArticle.aspx?ArticleID=177251
