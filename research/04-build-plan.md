# Build plan — household agent, optimised for cash prizes

Nov 20, 2026. Hack 11:00 → submit 4:30 PT. Derived from the sponsor files, `reference-projects/README.md`, and the organizer's Discord answers.

## Prize money, by sponsor

| Sponsor | Cash or cash-like | Paid places | Decision |
|---|---|---|---|
| SingleStore | $2,250 gift cards ($1,000 / $750 / $500) | 3 | **Core** |
| Airbyte | $1,750 Visa ($1,000 / $500 / $250) | 3 | **Core** |
| Amplitude + Statsig | $1,500 ($1,000 / $500) | 2 | **Core** |
| Firedrill | $1,000 Visa ($500 / $300 / $200) + credits | 3 | **Core** |
| Superlinked SIE | $1,000 cash ($500 × two tracks) + credits | 1 per track | **Core** — one integration competes for both |
| Cognee | $750 cash ($500 / $250) | 2 | **Drop** — its job (memory) is already SingleStore's; adding it dilutes "integral" |
| Persona | $250 Keychron card *per member* (~$1,000 for 4) | 1 | Optional, not cash, ~2 h |
| Senso | credits only | 3 | **Thin** — policy citations with `version_id`; organizer-aligned track |
| Guild, Arcee | tokens / credits / merch | — | Skip |

Five core sponsors, about $7,500 in cash-like prizes on the table.

## Rule: every sponsor owns a stage

Removal test: if a sponsor can be deleted and the demo still works, it is bolted on. Each one below owns a stage that the agent cannot run without.

```
 INGEST            UNDERSTAND           DECIDE + RECORD          ACT                  MEASURE / IMPROVE
 Airbyte      →    Superlinked SIE  →   SingleStore         →    Approval + order  →  Amplitude + Statsig
 Gmail receipts    extract line items   inventory, run-out SQL   one-tap message       sessions, funnel, v1/v2
 budget sheet      rerank substitutes   hybrid product search    Stripe test mode      Firedrill: simulated
 (calendar*)       Decide = approve?    order + audit row, 1 tx  idempotency key       housemates, red→green
```

| Stage | Sponsor | What it does (integral part) | Depth that impresses (from research) |
|---|---|---|---|
| Ingest | **Airbyte** | Reads each housemate's Gmail receipts and the shared budget sheet through the context layer (hosted MCP, read-only). This is the only source of purchase history — no manual tracking. | Every read logged as an audit row. Stretch: create the Stripe test-mode Checkout Session through `airbyte-agent-sdk`, so Airbyte both reads and writes. |
| Understand | **Superlinked SIE** | Small models as agent tools: `gliner2` extracts item, quantity, price from receipts with span checks; `Qwen3-Reranker` ranks substitutes after hard filters; `GLiNER2.5-Decide` is the "needs approval?" gate. A frontier model only plans. | Per-step ledger of model, latency, cost, evidence. That is both the "harness" and the "technical build" story. Warm models before the demo (cold starts take minutes). |
| Decide + record | **SingleStore** | The only database. Inventory, days-of-supply trigger SQL, hybrid full-text + vector product search, append-only event table, and the order plus its audit row committed in one transaction. | Live dashboard over agent decisions; the "Why" panel is a SQL query. Matches their CPO's "post-action auditability" post. |
| Measure | **Amplitude + Statsig** | Statsig holds the spend limit and agent version (v1/v2) as config. Amplitude AI SDK records each session, turn and tool call; `Order Placed` funnel split by variant; rejection stages as a funnel. | Using both together, in the pattern their docs show. Seed sessions early (15–20 min lag). |
| Improve | **Firedrill** | Simulated housemates drive `POST /chat`: one approves, one changes their mind, one pushes past the budget, one never replies. Synthetic Stripe with an exact check of one charge; payments outage mid-run. | Their brief verbatim: "show us what you learned and how you used it to improve your agent." Failing report → fix → passing re-run. |
| Policy citations | Senso (thin) | Store and delivery policies; each online-vs-pickup decision cites a passage with `version_id`. | Only if time allows beyond ~45 min. |

\* **Calendar source is unresolved.** The Airbyte research does not list a Google Calendar agent connector. Check at the access gate. If absent, read calendars directly with the Google Calendar API and keep Airbyte on receipts + budget. Firedrill has a synthetic Google Calendar, so drills can still include calendar events.

## Architecture rules (lock before writing code)

- `POST /chat` with `{"message","session"}` → `{"reply"}`, plus `GET /health`, port from env — Firedrill drives this.
- Payment URL from an env var — Firedrill substitutes synthetic Stripe.
- Every order carries an idempotency key — no double charge after an outage.
- One append-only `events` table in SingleStore → ledger, "Why" panel, Amplitude forwarding, cached demo replay.
- Model proposes, code verifies: budget, quantity, approver, delivery date recomputed in SQL before any order; "cannot measure" instead of guessing.
- Every screen that touches payment says **TEST MODE**.

## Owners (4 people)

| Person | Owns | Sponsors |
|---|---|---|
| A | Data + ledger: schema, trigger SQL, hybrid search, transaction, dashboard | SingleStore |
| B | Ingest + understanding: Airbyte reads, SIE extraction/rerank/decide | Airbyte, Superlinked |
| C | Agent loop + approval message + order path; `/chat` endpoint | (glue) |
| D | Measurement + simulation: Statsig, Amplitude, Firedrill drills, demo video | Amplitude/Statsig, Firedrill |

## Timeline

| Time | Milestone |
|---|---|
| 11:00–11:45 | **Access gate.** Each owner proves a hello-world call: SingleStore connected, Airbyte context-layer read works on the trial, SIE model warm, Firedrill agent registered, first Amplitude event visible, Statsig gate read. **Anything still failing at 11:45 is dropped, not debugged.** |
| 11:45–1:30 | Each stage works alone on seeded data. Endpoint up for Firedrill. |
| 1:30 | Lunch; first end-to-end run (party invite → plan → approval → test order → ledger). |
| 2:00 | First Firedrill run; pick the most interesting real failure. |
| 3:00 | Fix in; same scenario re-run green. Statsig v1 vs v2 compared in Amplitude. |
| 3:30 | **Record the video.** |
| 3:30–4:15 | README with a "How we used X" section per sponsor (file paths), screenshots. |
| 4:30 | Submit. |

## Demo — 3 minutes, every sponsor on screen

| Time | Beat | Sponsor on screen |
|---|---|---|
| 0:00 | Statistic; house dashboard — "nobody typed any of this" | Airbyte (receipts → inventory) |
| 0:25 | Receipt line items extracted, with the evidence span highlighted | Superlinked SIE |
| 0:45 | Party invite arrives; plan = needs − stock; run-out dates from SQL | SingleStore |
| 1:10 | Over the spend limit → one-tap approval message → Approve → test order + ledger row in one transaction | SingleStore, Statsig (limit) |
| 1:40 | "Why" panel: sources, checks, approver | SingleStore (+ Senso citation if built) |
| 2:00 | Firedrill: red report (e.g. double charge after outage) → fix → green | Firedrill |
| 2:30 | v1 vs v2 funnel and session view | Amplitude + Statsig |
| 2:50 | The order the agent refused to place; close | — |

## Fallback if one team can win only one track

Ask the organizers on Discord now. If the answer is one prize per team, go deepest on **SingleStore** (largest pool, three places) and let every other sponsor feed it; keep Firedrill as the demo's improvement story regardless.
