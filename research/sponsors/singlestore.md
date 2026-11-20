# SingleStore — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (Nov 20 2026, SF). Challenge: "build a shopping agent with an audit trail".
> Every non-obvious claim has a source URL. Anything I could not confirm is marked **unverified** or **not found**.
> SQL and code blocks are marked either **verbatim from docs** or **adapted (untested)**.

---

## 1. TL;DR

- **What it is:** SingleStore (formerly MemSQL) is a distributed, MySQL-wire-compatible SQL database that runs transactions, analytics, full-text search (Lucene/BM25) and vector search (native `VECTOR` type + ANN indexes) in one engine. The managed cloud is called **Helios**. ([docs llms.txt](https://docs.singlestore.com/llms.txt))
- **What the prize wants:** "Best Use of SingleStoreDB" — $1,000 / $750 / $500 gift cards. No rubric is published ([event page](https://tokensand.com/personalagenthack)). The last published SingleStore rubric (Cal Hacks 11.0) scored **Speed** (real-time data), **Analytics** (dashboards, RAG, agent workflows on that data), **Architecture** (SingleStore as *the* underlying database) and **Innovation** ([Cal Hacks 11.0 Devpost](https://cal-hacks-11-0.devpost.com/)).
- **Their own current pitch matches the challenge almost word for word.** SingleStore's CPO wrote on Oct 27 2026 that a database for agents needs "post-action auditability: reconstruct the exact data an agent read before it committed an action", plus check-and-act in one ACID transaction (his example: checking and reserving inventory) ([blog](https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/)). Build exactly that.
- **Fastest qualifying integration (about 30–45 min):** sign up at portal.singlestore.com (Google login), create a free **Shared** cluster, `pip install singlestoredb`, create a `products` table with a `VECTOR` column and a `FULLTEXT USING VERSION 2` index, and an `agent_events` ledger table with a `JSON` column. Run one hybrid (full-text + vector, RRF) query for product retrieval and write every agent step to the ledger.
- **What will separate a winner:** SingleStore is the *only* datastore — catalog, vectors, keyword search, cart/order transactions, the audit ledger and a live analytics view over agent decisions all in one database, with the purchase and its audit row committed in the same transaction. Past winners that only used it as a plain user table won small student prizes; the stronger ones showed live data plus analytics on it.

---

## 2. What they do

**Product.** A distributed SQL database ("HTAP": hybrid transactional/analytical) with:

- Scale-out architecture: aggregator nodes route queries, leaf nodes store data in partitions ([llms.txt](https://docs.singlestore.com/llms.txt)).
- Two storage engines: in-memory rowstore and disk-based columnstore ("Universal Storage" combines them) ([llms.txt](https://docs.singlestore.com/llms.txt)).
- Multi-model: relational, JSON (automatically columnarized), vectors, geospatial, time-series, key-value ([llms.txt](https://docs.singlestore.com/llms.txt), [Using JSON](https://docs.singlestore.com/cloud/create-a-database/using-json/)).
- Vector search: native `VECTOR(n [, F16|F32…])` type, exact kNN or ANN indexes (`AUTO, FLAT, IVF_FLAT, IVF_PQ, IVF_PQFS, HNSW_FLAT, HNSW_PQ`; SingleStore recommends `IVF_PQFS` and `HNSW_FLAT`) ([Vector Indexing](https://docs.singlestore.com/cloud/reference/sql-reference/vector-functions/vector-indexing/)).
- Full-text search "Version 2": Lucene-compatible, BM25 scoring ([Full-Text Search](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-full-text-search/)).
- Hybrid search is done in plain SQL: two CTEs (full-text, vector), `RANK()` each, `FULL OUTER JOIN`, reciprocal rank fusion ([Hybrid Search](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/hybrid-search-reranking-full-text-and-vector-search-results/)).
- **Pipelines**: built-in streaming/batch ingest from Kafka, S3 etc. ([llms.txt](https://docs.singlestore.com/llms.txt)).
- MySQL wire protocol on port 3306; HTTP **Data API** (JSON-based SQL over HTTPS) ([Data API](https://docs.singlestore.com/cloud/reference/data-api/)).

**Around the database (Helios cloud):**

| Feature | What it is | Status / note | Source |
|---|---|---|---|
| Notebooks | Hosted SQL + Python notebooks in the Cloud Portal | Listed as a connect option for Shared clusters | [Getting started](https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/) |
| Aura | Container service that hosts notebooks, AI functions, cloud functions, models | — | [AI & ML Functions](https://docs.singlestore.com/cloud/ai/ai-ml-functions/) |
| AI Functions | SQL functions: `AI_COMPLETE`, `AI_SENTIMENT`, `AI_TRANSLATE`, `AI_SUMMARIZE`, `AI_CLASSIFY`, `AI_EXTRACT`, `EMBED_TEXT`; called as `cluster.FUNC(...)`. Default LLM is Claude family; a shared embedding model `shared-qwen3-embed-0-6b` (1024 dims) is provisioned on install | Must be installed per cluster group; **availability on the free Shared tier is unverified** | [AI Functions](https://docs.singlestore.com/cloud/ai/ai-ml-functions/ai-functions/), [Shared Embedding Model Service](https://docs.singlestore.com/cloud/ai/ai-ml-functions/shared-embedding-model-service/) |
| Analyst (was "Aura Analyst") | Managed agentic text-to-SQL analyst; needs a configured "Domain" | Release notes: launched June 2026; blog post Sep 9 2026 | [Analyst docs](https://docs.singlestore.com/cloud/ai/analyst/), [release notes](https://docs.singlestore.com/cloud/release-notes/), [blog](https://www.singlestore.com/blog/introducing-singlestore-analyst-question-to-insight-while-it-still-matters/) |
| Models | Hosted/proxied LLMs and embedding models (Bedrock: Claude Opus 4.6 / Sonnet 4.6; Azure: gpt-5.4, gpt-4.1-mini, text-embedding-3-small/large; Aura-hosted Llama 3.2, Qwen) | — | [Models](https://docs.singlestore.com/cloud/ai/models/) |
| MCP server | Open-source (MIT) `singlestore-mcp-server`; lets Claude Code / Cursor / VS Code etc. list clusters, run SQL, create Shared clusters, create notebooks and scheduled jobs | First announced Mar 2025; repo last pushed Jun 2026 | [MCP docs](https://docs.singlestore.com/cloud/ai/singlestore-mcp-server/), [repo](https://github.com/singlestore-labs/mcp-server-singlestore), [blog](https://www.singlestore.com/blog/presenting-singlestore-mcp-server/) |
| Cloud Functions | Publish a shared notebook as a serverless HTTP endpoint | Preview | [Cloud Functions](https://docs.singlestore.com/cloud/container-services/cloud-functions/) |
| Kai | MongoDB-compatible API | **Maintenance mode — "SingleStore does not recommend Kai for new workloads"**. Do not build on it. | [Kai docs](https://docs.singlestore.com/cloud/reference/singlestore-kai/) |
| Observability Agent / Query Tuning Agent | Built-in DBA agents | Sep 28 and Sep 30 2026 (preview for Query Tuning) | [blog](https://www.singlestore.com/blog/introducing-observability-agent-gets-you-from-symptom-to-cause-in-minutes/), [blog](https://www.singlestore.com/blog/introducing-the-query-tuning-agent/) |

**Framework integrations.**

- LangChain: `pip install -U langchain-singlestore` — `SingleStoreVectorStore` (hybrid search strategies `VECTOR_ONLY`, `TEXT_ONLY`, `FILTER_BY_TEXT`, `FILTER_BY_VECTOR`, `WEIGHTED_SUM`), `SingleStoreChatMessageHistory`, `SingleStoreSemanticCache`, `SingleStoreSQLDatabaseRetriever`, `SingleStoreLoader` ([docs](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/connect-with-langchain/)).
- LangGraph: `singlestore-labs/langgraph-checkpointer-singlestore` (agent state checkpointer; small repo, last pushed Apr 2026) ([repo](https://github.com/singlestore-labs/langgraph-checkpointer-singlestore)).
- LlamaIndex: `SingleStoreVectorStore` in `llama_index.vector_stores.singlestoredb` ([LlamaIndex API ref](https://docs.llamaindex.ai/en/stable/api_reference/storage/vector_store/singlestoredb)).
- Drizzle ORM supported for Node ([Node docs](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/)).

**Positioning.** "One engine instead of a patchwork": replace OLTP DB + warehouse + search engine + vector DB + the pipelines between them. Judge Ryan Sarginson's Jan 2026 post argues exactly this and says agents "expose this fragility fastest" ([blog](https://www.singlestore.com/blog/singlestore-vs-the-classic-data-stack-why-real-time-and-ai-break-patchwork-architectures/)). VP Product Dave Eyler: SingleStore is "the engine for agentic applications" ([press release, Oct 1 2025](https://www.singlestore.com/media-hub/releases/singlestore-drops-three-major-upgrades-to-power-the-next-decade-of-ai-applications/)).

**Competitors** (named in SingleStore's own material): Postgres and MongoDB (as "rebranded AI databases"), ClickHouse, Snowflake/Snowpipe, BigQuery, Elasticsearch, and standalone vector databases ([CPO blog](https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/), [Sarginson blog](https://www.singlestore.com/blog/singlestore-vs-the-classic-data-stack-why-real-time-and-ai-break-patchwork-architectures/), ["Why your vector database should not be a vector database"](https://www.singlestore.com/blog/why-your-vector-database-should-not-be-a-vector-database/)).

---

## 3. Company facts

- **Founded** January 2011 as MemSQL by Eric Frenkiel, Nikita Shamgunov and Adam Prout; renamed SingleStore on Oct 27 2020. HQ San Francisco. About 380 employees (Wikipedia figure; the article is flagged for promotional content) ([Wikipedia](https://en.wikipedia.org/wiki/SingleStore)).
- **Leadership:** Raj Verma, CEO ([buyout close blog](https://www.singlestore.com/blog/a-new-chapter-for-singlestore-accelerating-our-growth-with-vector-capital/)). Edward Hsu joined as Chief Product Officer, announced Aug 3 2026 ([blog](https://www.singlestore.com/blog/welcoming-edward-hsu-as-singlestores-chief-product-officer/)).
- **Ownership: private-equity owned.** Vector Capital announced a majority "growth buyout" on Sep 10 2025 ([Business Wire](https://www.businesswire.com/news/home/20250910856970/en/SingleStore-Announces-Growth-Buyout-Led-by-Vector-Capital)); SingleStore said it closed on Oct 16 2025 ([blog](https://www.singlestore.com/blog/a-new-chapter-for-singlestore-accelerating-our-growth-with-vector-capital/)). Terms undisclosed; Bloomberg reported about $500M, well below the earlier $1.3B peak valuation ([Blocks & Files](https://www.blocksandfiles.com/ai-ml/2025/09/17/singlestore-sidesteps-into-private-equity-ownership/1589537), [TechTarget](https://www.techtarget.com/data-technologies/news/366630612/SingleStore-agrees-to-growth-buyout-by-private-equity-firm)). Dell Technologies Capital, Google Ventures, IBM and REV Venture Partners stayed as investors (TechTarget, same link).
- **Prior funding:** sources disagree — Wikipedia totals $318.1M; TechTarget says $464.1M before the buyout. Rounds include Series E $80M (Dec 2020, Insight), Series F $80M (Sep 2021), $116M (Jul 2022, Goldman Sachs AM), F-2 $30M (Oct 2022, Prosperity7) ([Wikipedia](https://en.wikipedia.org/wiki/SingleStore)).
- **Financials (company-reported):** over $123M ARR in Q2 FY2026, net-new ARR up 34% YoY ([press release, Oct 1 2025](https://www.singlestore.com/media-hub/releases/singlestore-drops-three-major-upgrades-to-power-the-next-decade-of-ai-applications/)).
- **Acquisition:** BryteFlow (Oct 2024), now "SingleStore Flow" for CDC ingest ([Wikipedia](https://en.wikipedia.org/wiki/SingleStore)).
- **Notable 2025–26 news:** AI & ML Functions, Zero Copy Attach and Aura Analyst announced Oct 1 2025 at SingleStore Now (press release above); Analyst launched June 2026 ([release notes](https://docs.singlestore.com/cloud/release-notes/)); PCI DSS v4.0.1 (Aug 3 2026), Observability Agent (Sep 28 2026), Query Tuning Agent preview (Sep 30 2026), `F16` vectors with ANN support (Oct 2026) ([blog index](https://www.singlestore.com/blog/), [release notes](https://docs.singlestore.com/cloud/release-notes/)).
- **Layoffs / distress news:** not found in searches.

---

## 4. Customers & use cases

Verified in this pass:

| Customer | Use | Source |
|---|---|---|
| 6sense | "500% faster, real-time customer insights at scale" (co-founder quote) | [press release](https://www.singlestore.com/media-hub/releases/singlestore-drops-three-major-upgrades-to-power-the-next-decade-of-ai-applications/) |
| Siemens | Vector capabilities on AWS for real-time decisions | [case study](https://www.singlestore.com/made-on/siemens/) |
| DirectlyApply | Job search "400x faster" using SingleStore vectors | [case study](https://www.singlestore.com/made-on/directlyapply/) |
| PYLER | Ad tech / brand safety | [case study](https://www.singlestore.com/made-on/pyler/) |
| Unnamed Fortune 25 bank | Vector insights on petabytes | [case study](https://www.singlestore.com/made-on/fortune25bank/) |
| DataVue | Phonetic match keys via Python UDFs | [blog, Sep 21 2026](https://www.singlestore.com/blog/from-two-codebases-to-one-how-datavue-brought-phonetic-match-keys-into-singlestore-with-python-udfs/) |

Verticals they write about most in late 2026: adtech (identity resolution, advertiser dashboards), fintech, iGaming compliance, healthcare ([blog index](https://www.singlestore.com/blog/)). Other well-known logos often associated with SingleStore were **not re-verified** here, so they are left out.

---

## 5. Why they are at this hackathon

Inferred from launches, blog cadence and prize wording:

1. **Reposition from "fast analytics DB" to "the database for AI agents".** Event tagline is "Real-time data for AI agents" / "Powering AI agents with real-time data" ([event page](https://tokensand.com/personalagenthack), [Luma](https://luma.com/personalagenthack)). The CPO's Oct 27 2026 post is a manifesto for it ([blog](https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/)).
2. **Developer adoption of the free Shared tier.** The three docs links they gave the organizers are: getting started with Helios, connect from Python, and working with vector data ([event page](https://tokensand.com/personalagenthack)). That is the funnel they want: sign up, connect, store vectors.
3. **Consolidation stories.** They want demos that show one database replacing Postgres + Pinecone + Elasticsearch + a warehouse. Both judges are Solutions Engineers — their day job is exactly this pitch to prospects.
4. **Field/SE-led, not DevRel-led.** They sent two SEs rather than developer advocates, and the prize is gift cards, not credits. Expect practical judges who will ask "what is actually in the database and what queries run against it?".

---

## 6. Prize track

**Exact wording** ([event page](https://tokensand.com/personalagenthack)):

> SingleStore — "Real-time data for AI agents." — **Best Use of SingleStoreDB** — 1st: $1,000 · 2nd: $750 · 3rd: $500 in gift cards of your choice

No judging criteria, workshop title or talk title is published for SingleStore on the event page or Luma page ([Luma](https://luma.com/personalagenthack)).

**People attending**

**Amanda Kwong — Solutions Engineer (judge and speaker)** ([Luma](https://luma.com/personalagenthack))
- GitHub `akwong-singlestore` (account created May 2026), about 94 pull requests, almost all to SingleStore's public demo assets ([GitHub PR search](https://github.com/search?q=author%3Aakwong-singlestore+type%3Apr&type=pullrequests)):
  - `singlestore-labs/spaces-notebooks` — the notebook gallery inside the Cloud Portal: Fundamentals notebook, cheat sheet, TPC-H 100, Confluent Cloud, "modern data pipelines", "marketing transactions sequences", YouTube walkthrough embeds ([example PR](https://github.com/singlestore-labs/spaces-notebooks/pull/198), [PR 228](https://github.com/singlestore-labs/spaces-notebooks/pull/228)).
  - `singlestore-labs/demo-realtime-digital-marketing` — real-time ad-serving demo; she added an **Aura Analyst chat integration** ([PR 94](https://github.com/singlestore-labs/demo-realtime-digital-marketing/pull/94)).
  - A personal repo comparing Postgres vs SingleStore on TPC-H SF100 ([repo PR](https://github.com/akwong-singlestore/tpch-sf100-postgres-vs-singlestore/pull/1)).
  - She removed the deprecated Kai section from the Fundamentals notebook ([PR 212](https://github.com/singlestore-labs/spaces-notebooks/pull/212)) — another reason not to use Kai.
- Read: she builds onboarding demos. She will value a clean, legible demo with live-updating charts, pipelines, and Analyst-style natural-language querying over real data. LinkedIn profile and prior background: **not found**.

**Ryan Sarginson — Solutions Engineer (judge)** ([Luma](https://luma.com/personalagenthack))
- SingleStore blog author page lists him as Solutions Engineer with six posts ([author page](https://www.singlestore.com/blog/author/ryan-sarginson/)); he was a Solutions Engineering Intern in 2024 (per search snippet of the same page — **the intern title is from a search result, not re-confirmed**).
- Posts: "Full-Text Search: Version 2" with Eric Hanson (Nov 21 2024) ([link](https://www.singlestore.com/blog/full-text-search-version-2/)); "Elastic's OLAP Weaknesses" (Oct 15 2024) ([link](https://www.singlestore.com/blog/elastic-olap-weaknesses/)); "How We Built Our Real-Time Digital Marketing App" (Aug 27 2024) ([link](https://www.singlestore.com/blog/how-we-built-our-real-time-digital-marketing-app/)); "Easy Integration of MongoDB and Elasticsearch" (Aug 21 2024) ([link](https://www.singlestore.com/blog/integrating-mongodb-and-elasticsearch/)); "SingleStore vs. the Classic Data Stack" (Jan 14 2026) ([link](https://www.singlestore.com/blog/singlestore-vs-the-classic-data-stack-why-real-time-and-ai-break-patchwork-architectures/)); "You Built Right. Now Build Bigger." on fintech Postgres migration (Jun 3 2026) ([link](https://www.singlestore.com/blog/fintech-postgres-migration-singlestore/)).
- Read: he co-wrote the launch post for full-text search V2 and argues against stack sprawl. A hybrid search query using `FULLTEXT USING VERSION 2` plus vectors, in a single-database architecture, is aimed squarely at him. Education and earlier career: **not found**.

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **One database for everything in the agent loop** — catalog, embeddings, keyword index, cart/orders, agent memory and the audit ledger. No Pinecone, no separate Postgres, no Redis. Say this on the architecture slide. (Matches the Cal Hacks "Architecture" criterion and both judges' writing.)
2. **Check-and-act in one ACID transaction with the audit row.** Read price/stock, decrement stock, insert order, insert ledger row, commit. This is the CPO's own example of what agents need ([blog](https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/)).
3. **Hybrid search in SQL** — `FULLTEXT USING VERSION 2` + `VECTOR` index + RRF + a SQL filter (price, in-stock) in one statement ([docs](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/hybrid-search-reranking-full-text-and-vector-search-results/)).
4. **Real-time analytics on the agent's own decisions** — a live dashboard that aggregates the ledger (steps per purchase, rejected candidates, savings vs list price, latency per tool call) seconds after rows land. "Speed" and "Analytics" criteria.
5. **JSON columns** for tool-call payloads and retrieved-context snapshots, queried with `::` path syntax ([Using JSON](https://docs.singlestore.com/cloud/create-a-database/using-json/)).
6. **Streaming ingest** (Pipelines from Kafka/S3, or just high-rate inserts of price/stock changes) so the agent decides on data that changed seconds ago.
7. **AI Functions / `EMBED_TEXT` in SQL, Analyst, MCP server, notebooks.** Nice extras; each is newer and less certain to be available on the free tier. Using the MCP server during development is free credit: mention it.
8. **Kai (Mongo API):** do not use. It is in maintenance mode ([docs](https://docs.singlestore.com/cloud/reference/singlestore-kai/)).

---

## 8. Quickstart

### 8.1 Signup (target: under 10 minutes)

1. Go to https://portal.singlestore.com/ and sign up with email, Google or Microsoft ([signup docs](https://docs.singlestore.com/cloud/developer-resources/try-singlestore-helios-for-free/sign-up-to-try-singlestore-helios/)).
2. Create a **Shared Edition** cluster. It "does not consume any credits" and has no inactivity auto-suspend (same link). A Standard trial with free credits is the alternative; the credit amount is not stated in the docs (a third-party site says $500 — **unverified**).
3. In the portal: **Clusters → your Shared cluster → Connect** to get host, user, password and ready-made snippets for Python, Node.js, HTTP Data API etc. ([getting started](https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/)).
4. Whether a credit card is required: **unverified** (not stated in the docs or pricing page).

### 8.2 Free Shared tier limits ([Shared Edition docs](https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/about-singlestore-helios/singlestore-helios-editions/shared-edition/))

| Limit | Value |
|---|---|
| Region | AWS US-EAST-1 only |
| Databases per shared cluster | 1 (`CREATE DATABASE` / `DROP DATABASE` are disabled) |
| **Tables per database** | **10** |
| Rowstore tables | None allowed (columnstore only) |
| Reference tables | None allowed |
| Pipelines | 10 (no CDC-in pipelines) |
| Stored procedures + UDFs | 10 |
| Data size | 1 GB compressed |
| Database users | 3, "admin or Read only" |
| Concurrent connections | 100 |
| Firewall | Not configurable; reachable from anywhere |
| Built-in audit logging | Not supported |
| Global variables | Cannot be changed |
| Stage (file storage) | Up to 10 GB |

**Gotchas**

- **SSL is required for connections to a Shared cluster** ([Node docs](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/)). CA bundle: https://portal.singlestore.com/static/ca/singlestore_bundle.pem ([endpoints docs](https://docs.singlestore.com/cloud/connect-to-singlestore/singlestore-helios-endpoints/)). Copy the exact connection snippet from the portal's Connect panel rather than hand-writing it.
- Budget your 10 tables up front.
- Vector indexes only work on columnstore tables and only on `F16`/`F32` vectors (fine: columnstore is the only kind allowed on Shared) ([Vector Indexing](https://docs.singlestore.com/cloud/reference/sql-reference/vector-functions/vector-indexing/)).
- A query only uses a vector index if it uses the same metric the index was built with (default `DOT_PRODUCT`). For `DOT_PRODUCT` to equal cosine similarity, vectors must be normalized to length 1. Order `DESC` for `<*>`, `ASC` for `<->` ([Vector Indexing](https://docs.singlestore.com/cloud/reference/sql-reference/vector-functions/vector-indexing/)).
- Newly inserted rows are indexed asynchronously; run `OPTIMIZE TABLE <t> FLUSH;` after loading so full-text and vector indexes see them ([Hybrid Search docs](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/hybrid-search-reranking-full-text-and-vector-search-results/)).
- Put `RANK()` in a separate CTE from the search CTE, otherwise the index is not used (same link).
- Only one full-text function (`MATCH`, `BM25`, `BM25_GLOBAL`) per query block, and each applies to one table ([Full-Text Search](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-full-text-search/)).
- Insert vectors as JSON-array strings for convenience (fine at hackathon scale; binary is recommended for production) ([Vector Data](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-vector-data/)).
- Data API request body limit is 1 MB ([Data API](https://docs.singlestore.com/cloud/reference/data-api/)).
- Docs for coding agents: append `.md` to any docs URL; index at https://docs.singlestore.com/llms.txt.

### 8.3 Python connection — verbatim from docs ([Python client](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/connect-using-the-singlestore-python-client/))

```bash
pip install singlestoredb
```

```python
import singlestoredb as s2
conn = s2.connect(host='...', port='...', user='...',
                  password='...', database='...')
```

```python
import singlestoredb as s2
conn = s2.connect('user:password@host:port/database')
```

```python
with conn.cursor() as cur:
    cur.execute('show variables like %s', ['auto%'])
    for row in cur.fetchall():
        print(row)
```

Parameters use `%s` (positional) or `%(name)s` (named); `results_type='dict'` returns dict rows. Python 3.9+. The same client can talk to the HTTP Data API with `s2.connect('https://user:password@host:port/database')` (same source).

### 8.4 Node.js connection — verbatim from docs ([Node docs](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/), [Node SSL](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/connect-with-node-js-using-ssl/))

```bash
npm install --save singlestore-nodejs
```

```javascript
import singlestore from "singlestore-nodejs/promise";

const connection = await singlestore.createConnection({
  host: "<hostname>",
  port: 3306,
  user: "<username>",
  password: "<password>",
  database: "<database_name>",
});
```

With SSL (required on Shared):

```javascript
const singlestore = require("singlestore-nodejs");
var fs = require("fs");

var connection = singlestore.createConnection({
    host: <hostname>,
    user: '<username>',
    password: <password>,
    ssl: {
        ca: fs.readFileSync(__dirname + '/singlestore_bundle.pem')
    }
});
```

Queries use `?` placeholders: `await conn.execute("SELECT * FROM users WHERE id = ?", [userId])` (same source).

### 8.5 SQL — verbatim from docs

Vector table and inserts ([Vector Data](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-vector-data/)):

```sql
CREATE TABLE comments(id INT,
   comment TEXT,
   comment_embedding VECTOR(4),
   category VARCHAR(256));

INSERT INTO comments VALUES
(1, "The cafeteria in building 35 has a great salad bar",
    '[0.45, 0.55, 0.495, 0.5]',
    "Food");
```

Similarity query with a metadata filter (`<*>` is `DOT_PRODUCT`, `<->` is `EUCLIDEAN_DISTANCE`):

```sql
SET @query_vec = ('[0.44, 0.554, 0.34, 0.62]'):>VECTOR(4);

SELECT id, comment, category,
         comment_embedding <*> @query_vec AS score
    FROM comments
    WHERE category = "Food"
    ORDER BY score DESC
    LIMIT 3;
```

Vector (ANN) index:

```sql
ALTER TABLE comments ADD VECTOR INDEX ivf(comment_embedding)
INDEX_OPTIONS '{"index_type":"IVF_PQFS"}';

OPTIMIZE TABLE comments FLUSH;
```

Confirm the index is used: `EXPLAIN <query>` should show `INTERNAL_VECTOR_SEARCH` in the `ColumnStoreFilter` row (same source). Readable vector output: `SET vector_type_project_format = JSON;`.

Full-text index in `CREATE TABLE` ([Full-Text Search](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-full-text-search/)):

```sql
CREATE TABLE articles (
    id INT UNSIGNED,
    year int UNSIGNED,
    title VARCHAR(200),
    body TEXT,
    SORT KEY (id),
    FULLTEXT USING VERSION 2 art_ft_index (title, body));
```

Hybrid search with RRF and a SQL filter ([Hybrid Search](https://docs.singlestore.com/cloud/developer-resources/functional-extensions/hybrid-search-reranking-full-text-and-vector-search-results/)):

```sql
ALTER TABLE comments ADD FULLTEXT USING VERSION 2 ft_comment(comment);

OPTIMIZE TABLE comments FLUSH;

SET @query_comment = 'comment:restaurant';
SET @query_vec = ('[0.44, 0.554, 0.34, 0.62]'):>VECTOR(4);

WITH fts AS ( /* Find top full-text matches. */
  SELECT
    id,
    comment,
    MATCH(TABLE comments) AGAINST (@query_comment) AS ft_score
  FROM comments
  WHERE MATCH(TABLE comments) AGAINST(@query_comment)
  ORDER BY ft_score desc
  LIMIT 200
),
vs AS ( /* Find top vector search matches. <*> is dot product. */
  SELECT
    id,
    comment,
    @query_vec <*> comment_embedding AS vec_score
  FROM comments
  WHERE category = "Food"
  ORDER BY vec_score desc
  LIMIT 200),
fts_ranked as (
  SELECT
    *,
    RANK() OVER (ORDER BY ft_score DESC) AS fts_rank
  FROM fts
),
vs_ranked as (
  SELECT
    *,
    RANK() OVER (ORDER BY vec_score DESC) as vs_rank
  FROM vs
)
SELECT
  *,
  0.7 * (1.0 / (NVL(fts_rank, 1000.0) + 60)) + 0.3 * (1.0 / (NVL(vs_rank, 1000.0) + 60)) AS combined_score
FROM fts_ranked f FULL OUTER JOIN vs_ranked v ON f.id = v.id
ORDER BY combined_score DESC
LIMIT 3;
```

In-database embedding, if AI Functions are installed on the cluster ([Shared Embedding Model Service](https://docs.singlestore.com/cloud/ai/ai-ml-functions/shared-embedding-model-service/)) — 1024 dimensions, so use `VECTOR(1024)`:

```sql
SET batch_external_functions = AUTO;

UPDATE customer_reviews
SET review_embedding = cluster.EMBED_TEXT(review_text, 'shared-qwen3-embed-0-6b')
WHERE review_embedding IS NULL;
```

LLM call in SQL ([AI Functions](https://docs.singlestore.com/cloud/ai/ai-ml-functions/ai-functions/)): `SELECT cluster.AI_COMPLETE('Life is like a box of') AS completion;`

LangChain shortcut — verbatim from docs ([LangChain docs](https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/connect-with-langchain/)); the docs' own example is a product recommender:

```python
from langchain_huggingface import HuggingFaceEmbeddings
from langchain_singlestore import SingleStoreVectorStore

embedding = HuggingFaceEmbeddings(
    model_name="sentence-transformers/all-MiniLM-L6-v2"
)

vectorstore = SingleStoreVectorStore(
    host="svchost",
    port=3306,
    user="s2user",
    password="passkey",
    database="dbTest",
    embedding=embedding
)

vectorstore.add_texts(products)
results = vectorstore.similarity_search(user_query, k=2)
```

Connection URL env var for LangChain/LlamaIndex: `SINGLESTOREDB_URL=singlestoredb://<username>:<password>@<host>:<port>/<database>` (same source).

MCP server for your coding agent ([MCP docs](https://docs.singlestore.com/cloud/ai/singlestore-mcp-server/)); needs Python 3.11+ and `uv`; auth is browser OAuth, no API key:

```bash
uvx singlestore-mcp-server init --client=claude-code
# or
claude mcp add singlestore-mcp-server uvx singlestore-mcp-server start
```

### 8.6 Shopping-agent schema — adapted from the doc patterns above (untested; verify on the cluster)

```sql
-- Table 1: catalog with keyword + vector search
CREATE TABLE products (
    id BIGINT,
    title VARCHAR(300),
    description TEXT,
    category VARCHAR(128),
    price DECIMAL(10,2),
    stock INT,
    attrs JSON,
    embedding VECTOR(1024),          -- match your embedding model's dimension
    SORT KEY (id),
    FULLTEXT USING VERSION 2 ft_products (title, description)
);
ALTER TABLE products ADD VECTOR INDEX ivf_products(embedding)
    INDEX_OPTIONS '{"index_type":"IVF_PQFS"}';
OPTIMIZE TABLE products FLUSH;

-- Table 2: append-only audit ledger
CREATE TABLE agent_events (
    event_id BIGINT,
    run_id VARCHAR(64),
    seq INT,
    ts DATETIME(6),
    step_type VARCHAR(32),           -- plan | retrieve | compare | decide | act | error
    tool VARCHAR(64),
    input JSON,                      -- tool arguments / query
    evidence JSON,                   -- ids, scores, ranks, prices of rows the agent read
    output JSON,
    mode VARCHAR(16),                -- real | simulated | test
    prev_hash CHAR(64),
    row_hash CHAR(64),
    SORT KEY (ts)
);
```

Python sketch (adapted, untested):

```python
import json, singlestoredb as s2
conn = s2.connect(host=HOST, port=3306, user=USER, password=PW, database=DB)

def log_event(cur, run_id, seq, step_type, tool, inp, evidence, out, mode, prev_hash, row_hash):
    cur.execute(
        "INSERT INTO agent_events (event_id, run_id, seq, ts, step_type, tool, input, evidence, output, mode, prev_hash, row_hash) "
        "VALUES (%s,%s,%s,NOW(6),%s,%s,%s,%s,%s,%s,%s,%s)",
        [event_id(), run_id, seq, step_type, tool,
         json.dumps(inp), json.dumps(evidence), json.dumps(out), mode, prev_hash, row_hash])
```

Open points to check on the real cluster: default autocommit behaviour of the Python client and explicit `BEGIN`/`COMMIT` for the buy step; whether a parameterised query vector still triggers the ANN index (the docs say the query vector "must be a runtime constant" — check with `EXPLAIN`); `AUTO_INCREMENT` behaviour on a distributed table (IDs are unique but not sequential — generate `seq` in the app).

---

## 9. Past hackathons SingleStore sponsored

**Recent SF AI-agent hackathons with a SingleStore prize: not found.** I searched Devpost, Luma, LinkedIn-indexed results and the SingleStore blog. tokens& events I could open (Ship to Prod, Harness Engineering Hack, Multimodal Frontier) do not list SingleStore as a sponsor ([Ship to Prod](https://luma.com/shiptoprod), [Harness Hack](https://luma.com/harnesshack), [Multimodal](https://luma.com/multimodalhack)). The premise that SingleStore sponsors many SF AI hackathons with public winner lists is **unverified**; the evidence I found is below.

### Cal Hacks 11.0 — Oct 2024, San Francisco ([Devpost](https://cal-hacks-11-0.devpost.com/))

- **Track:** "Singlestore: Best Real-Time App" — 1 winner, 4x AirPods Pro 2.
- **Published criteria (verbatim):** "Speed: Process and utilize data in real-time (~milliseconds latency), eg. video streaming, voice agents, trading simulations, IoT sensor data. Analytics: Show us what the data means, and actions that you can take based on these insights via real-time dashboards, RAG, agent-based workflows, etc. Architecture: Use SingleStore as the underlying database. Innovation: Show us something cool and be creative!"
- **Winner: CryptoWise — Intelligent Trading and Insights Platform** ([Devpost](https://devpost.com/software/cryptowise-intelligent-trading-and-insights-platform)). Live crypto prices arrive over WebSocket APIs and are stored in SingleStore "for fast retrieval and analytics"; charts built on that data; an LLM chatbot (Groq, OpenAI) answers questions over real-time and historical data; a predictive model suggests Buy/Sell/Hold and can execute trades automatically.
- Also at this event (won a *different* sponsor's prize, but shows a strong SingleStore pattern): **FLEX** ([Devpost](https://devpost.com/software/flex-se8mtf)) — Fetch.ai agents, one of which queries SingleStore using "real-time data analysis and Full-Text Search" to match freelancers.

### SB Hacks XI — Jan 2025, UC Santa Barbara ([Devpost](https://sb-hacks-xi.devpost.com/))

- **Track:** "Best use of SingleStore" — 1 winner, $500 cash.
- **Winner: UCSBites** ([Devpost](https://devpost.com/software/ucsbites)) — matches students at dining halls. React front end, Flask backend; "We ran a cluster on SingleStore and used it as a database for our users". A first-time team; plain CRUD use.

### SingleStore Database Hackathon — Oct 3 – Nov 7 2022, online, $30K pool, 535 participants ([Devpost](https://singlestore.devpost.com/), [gallery](https://singlestore.devpost.com/project-gallery))

| Award | Project | What it did / how it used SingleStore |
|---|---|---|
| Multi-Model Champion ($5,000 per search snippet of the Devpost prize list) | [Animal Explorer](https://devpost.com/software/animal-explorer) | Node/Express + Leaflet map; draw an area and find animals whose range intersects it, using SingleStore geospatial data |
| HTAP Champion | [Router Holmes](https://devpost.com/software/drrouter) | Simulated city-wide router telemetry → Kafka/Confluent Cloud → **SingleStore Pipeline** → live map analytics polled from the DB |
| ML & AI Champion | [Movie Recommender](https://devpost.com/software/movie-recommender-67mjl2) | ALS factors stored per user and movie in SingleStore; recommendations computed with SQL; Streamlit UI |
| WASM Champion | [Drone surveillance/delivery on SingleStore](https://devpost.com/software/drone-surveillance-delivery-on-singlestore) | "AI based path optimization inside of SingleStore" (Wasm UDFs) |
| Database Migration Champion | [Airbyte SingleStore Connector](https://devpost.com/software/airbyte-singlestore-connector) | Airbyte destination connector for SingleStore |
| Honorable mentions | [Brave Chat](https://devpost.com/software/brave-chat), [Copyable](https://devpost.com/software/singlestore-migration-tool), [Reserva](https://devpost.com/software/reserva), [wasm-udf-quickjs-eval](https://devpost.com/software/wasm-udf-quickjs-eval), [the journal app](https://devpost.com/software/journal-u2m0er) | Slack alternative; migration SaaS; CBDC API "with HTAP"; JS-eval Wasm UDF; encrypted journal |

SingleStore's own winners post (Nov 23 2022) now returns 404; a third-party summary survives ([Plushcap](https://www.plushcap.com/content/singlestore/blog/singlestore-singlestore-hackathon-2022-big-winners)).

### SingleStore x Data Science UCSB x Coders SB Datathon — Jan 25–30 2024, online ([Devpost](https://singlestore-ucsb-datathon-2024.devpost.com/))

Wine-quality dataset projects; the gallery shows no winner badges — **winners not found**.

### Other

- "Build a Real-Time App with SingleStore and WIN" (SingleStore Now 2025 contest) appeared in search results but the page now returns 404 — **details not found** (https://www.singlestore.com/singlestore-now-contest-2025).

### Patterns in what wins

1. **Live data flowing in during the demo.** CryptoWise (WebSocket price feed) and Router Holmes (Kafka → Pipeline) both showed data arriving and the UI changing in real time.
2. **Insight plus action on that data.** The Cal Hacks rubric literally asks for "actions that you can take based on these insights"; the winner auto-executed trades. A shopping agent that buys on fresh price/stock data is the same shape.
3. **SingleStore as the system of record, not a sidecar.** "Architecture: Use SingleStore as the underlying database."
4. **Using a distinctive feature by name** won the category prizes in 2022: geospatial, Pipelines, Wasm UDFs. Today's equivalents are vector + full-text hybrid search, JSON, Pipelines, AI Functions.
5. **The bar at student events was low** (UCSBites: a users table). The SE judges at this event will likely expect more, but it shows that simply being the team that most clearly builds *on* SingleStore counts for a lot.

---

## 10. Fit with "shopping agent with an audit trail"

Table budget on the free tier is 10; the designs below use 3–6.

| # | Idea | What you build | Time in a 5.5h build | Impresses because |
|---|---|---|---|---|
| A | **Ledger only (shallow)** | `agent_events` append-only table with JSON `input` / `evidence` / `output`; every plan, retrieval, comparison, decision and action writes a row; a "trail" page reads it back by `run_id` | 30 min | Qualifies; JSON + SQL; satisfies challenge point 3 |
| B | **Catalog + hybrid retrieval** | `products` with `VECTOR` + `FULLTEXT USING VERSION 2`; one RRF query with price/stock filters; store returned ids, ranks and scores in the ledger's `evidence` | +60–75 min (embedding the catalog is the slow part) | Ryan co-wrote the FTS V2 launch post; "reliable context" becomes provable: the trail shows exactly which rows and scores the agent saw |
| C | **Transactional purchase with evidence snapshot** | `orders`, `cart_items`, stock on `products`. One transaction: re-read price and stock, decrement stock, insert order, insert ledger row with the price/stock values read. Demo a race: two agents, one item left, only one succeeds and the loser's trail says why | +45–60 min | Directly demonstrates the CPO's "check and action read the same data" and "post-action auditability" claims ([blog](https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/)) |
| D | **Live analytics over agent decisions** | A dashboard polling aggregate SQL over `agent_events` + `orders`: steps per run, tool latency p95, candidates rejected and why, savings vs list price, share of actions that were real vs simulated. Add a background script that mutates prices/stock so numbers move on screen | +45–60 min | "Speed" + "Analytics" from the Cal Hacks rubric; mirrors the real-time demo both judges have worked on |
| E | **Deep: in-database AI + tamper-evident ledger + ask-the-trail** | (i) `cluster.EMBED_TEXT` for embeddings and `AI_CLASSIFY`/`AI_EXTRACT` over reviews in SQL, if AI Functions are available on your cluster; (ii) hash-chain each ledger row (`prev_hash`, `row_hash`, computed in app) and show a "verify chain" button; (iii) natural-language questions over the ledger ("why did you skip the cheaper one?") via text-to-SQL or Analyst; (iv) agent memory/preferences table with vectors | +60–90 min, higher risk | Shows the newest features; the hash chain makes "audit trail" literal |

**Recommended path:** A → B → C → D (about 3–3.5 hours of the 5.5), with E(ii) hash chain as a cheap add-on (20 min) and E(i)/(iii) only if AI Functions work on the free tier within 10 minutes of trying.

**Demo script for the SingleStore judges (60 seconds):**
1. Architecture slide: one box labelled SingleStore; list what it replaced (vector DB, search engine, OLTP DB, analytics store, log store).
2. Show the hybrid SQL query and `EXPLAIN` with `INTERNAL_VECTOR_SEARCH`.
3. Trigger a price change live; the agent's next decision changes; the dashboard updates.
4. Open the audit trail for a purchase: the exact rows, scores, price and stock the agent read, the transaction that bought it, and the real/simulated flag.
5. Run one aggregate query over all agent runs.

**How this stacks with other sponsor tracks:** the ledger table is a natural sink for Senso retrieval fields (`content_id`, `version_id`, `score`, `rank` — the event brief says to log these with every decision), Superlinked/SIE embeddings can populate the `VECTOR` column, and Airbyte can be the ingest path into SingleStore.

---

## 11. Risks and open questions for the SingleStore booth

**Risks**

- Signup friction at the venue (email verification, possible card prompt — unverified). Create the account and Shared cluster the night before.
- Shared tier is multi-tenant in US-EAST-1; latency from SF and noisy neighbours are possible. It is "not intended for benchmarking" ([Shared Edition](https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/about-singlestore-helios/singlestore-helios-editions/shared-edition/)) — do not make millisecond claims you have not measured.
- 10-table and 10-procedure limits.
- Shared-tier users are only "admin or Read only", so you cannot create an insert-only DB role to enforce append-only at the permission level; use a read-only user for the audit viewer plus a hash chain.
- AI Functions, Analyst and Cloud Functions may not be available (or may need a paid/Standard cluster) — unverified.
- Async index flush: freshly inserted products may not show up in full-text/ANN results until `OPTIMIZE TABLE ... FLUSH`.
- Embedding dimension mismatch between your model and the `VECTOR(n)` column is a hard error; pick the model first.
- Do not use Kai.

**Questions to ask Amanda and Ryan**

1. What are the judging criteria for "Best Use of SingleStoreDB"? Same four as Cal Hacks (speed, analytics, architecture, innovation)?
2. Are AI Functions (`EMBED_TEXT`, `AI_COMPLETE`) and Analyst available on the free Shared cluster? If not, are hackathon credits or a Standard cluster available?
3. Is there a hackathon promo code / credits, and is a credit card needed?
4. For an audit ledger on a columnstore table: recommended `SHARD KEY` / `SORT KEY`, and do multi-statement transactions across `products`, `orders` and the ledger behave as expected on Shared?
5. Does a parameterised query vector from the Python client use the ANN index, or must it be a `SET @v = ...:>VECTOR(n)` session variable?
6. Best way to push live updates to a dashboard — poll, or is there a change-stream option?
7. Is there a recommended agent-memory pattern or repo (LangGraph checkpointer, MCP server) they would like to see used?
8. Do they prefer raw SQL in the demo or is the LangChain `SingleStoreVectorStore` fine?

---

## 12. Sources

Event
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack
- https://luma.com/shiptoprod · https://luma.com/harnesshack · https://luma.com/multimodalhack

SingleStore docs
- https://docs.singlestore.com/llms.txt
- https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/
- https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/about-singlestore-helios/singlestore-helios-editions/shared-edition/
- https://docs.singlestore.com/cloud/developer-resources/try-singlestore-helios-for-free/sign-up-to-try-singlestore-helios/
- https://docs.singlestore.com/cloud/connect-to-singlestore/singlestore-helios-endpoints/
- https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/
- https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/connect-using-the-singlestore-python-client/
- https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/connect-with-langchain/
- https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/
- https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-node-js/connect-with-node-js-using-ssl/
- https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-vector-data/
- https://docs.singlestore.com/cloud/developer-resources/functional-extensions/hybrid-search-reranking-full-text-and-vector-search-results/
- https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-full-text-search/
- https://docs.singlestore.com/cloud/reference/sql-reference/vector-functions/vector-indexing/
- https://docs.singlestore.com/cloud/create-a-database/using-json/
- https://docs.singlestore.com/cloud/reference/data-api/
- https://docs.singlestore.com/cloud/reference/singlestore-kai/
- https://docs.singlestore.com/cloud/ai/
- https://docs.singlestore.com/cloud/ai/ai-ml-functions/
- https://docs.singlestore.com/cloud/ai/ai-ml-functions/ai-functions/
- https://docs.singlestore.com/cloud/ai/ai-ml-functions/shared-embedding-model-service/
- https://docs.singlestore.com/cloud/ai/analyst/
- https://docs.singlestore.com/cloud/ai/models/
- https://docs.singlestore.com/cloud/ai/singlestore-mcp-server/
- https://docs.singlestore.com/cloud/ai/singlestore-ai-rules/
- https://docs.singlestore.com/cloud/container-services/cloud-functions/
- https://docs.singlestore.com/cloud/release-notes/
- https://www.singlestore.com/cloud-pricing/

SingleStore blog / press
- https://www.singlestore.com/blog/
- https://www.singlestore.com/blog/what-a-database-for-ai-agents-needs-is-already-clear/
- https://www.singlestore.com/blog/singlestore-vs-the-classic-data-stack-why-real-time-and-ai-break-patchwork-architectures/
- https://www.singlestore.com/blog/author/ryan-sarginson/
- https://www.singlestore.com/blog/full-text-search-version-2/
- https://www.singlestore.com/blog/a-new-chapter-for-singlestore-accelerating-our-growth-with-vector-capital/
- https://www.singlestore.com/blog/presenting-singlestore-mcp-server/
- https://www.singlestore.com/blog/introducing-singlestore-analyst-question-to-insight-while-it-still-matters/
- https://www.singlestore.com/blog/introducing-observability-agent-gets-you-from-symptom-to-cause-in-minutes/
- https://www.singlestore.com/blog/introducing-the-query-tuning-agent/
- https://www.singlestore.com/blog/welcoming-edward-hsu-as-singlestores-chief-product-officer/
- https://www.singlestore.com/media-hub/releases/singlestore-drops-three-major-upgrades-to-power-the-next-decade-of-ai-applications/
- https://www.singlestore.com/customers/ (and /made-on/siemens/, /made-on/directlyapply/, /made-on/pyler/, /made-on/fortune25bank/)

Company / news
- https://en.wikipedia.org/wiki/SingleStore
- https://www.businesswire.com/news/home/20250910856970/en/SingleStore-Announces-Growth-Buyout-Led-by-Vector-Capital
- https://www.techtarget.com/data-technologies/news/366630612/SingleStore-agrees-to-growth-buyout-by-private-equity-firm
- https://www.blocksandfiles.com/ai-ml/2025/09/17/singlestore-sidesteps-into-private-equity-ownership/1589537

GitHub
- https://github.com/singlestore-labs/mcp-server-singlestore
- https://github.com/singlestore-labs/langchain-singlestore
- https://github.com/singlestore-labs/langgraph-checkpointer-singlestore
- https://github.com/singlestore-labs/spaces-notebooks/pull/198 · /pull/212 · /pull/228
- https://github.com/singlestore-labs/demo-realtime-digital-marketing/pull/94
- https://docs.llamaindex.ai/en/stable/api_reference/storage/vector_store/singlestoredb

Hackathons
- https://cal-hacks-11-0.devpost.com/ · https://devpost.com/software/cryptowise-intelligent-trading-and-insights-platform · https://devpost.com/software/flex-se8mtf
- https://sb-hacks-xi.devpost.com/ · https://devpost.com/software/ucsbites
- https://singlestore.devpost.com/ · https://singlestore.devpost.com/project-gallery
- https://devpost.com/software/animal-explorer · https://devpost.com/software/drrouter · https://devpost.com/software/movie-recommender-67mjl2 · https://devpost.com/software/drone-surveillance-delivery-on-singlestore · https://devpost.com/software/airbyte-singlestore-connector
- https://singlestore-ucsb-datathon-2024.devpost.com/
- https://www.plushcap.com/content/singlestore/blog/singlestore-singlestore-hackathon-2022-big-winners
