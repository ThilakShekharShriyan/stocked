# Personal Agents Hackathon — Research Index

**Event:** Friday Nov 20, 2026 · San Francisco · hack 11:00 AM → submit 4:30 PM PT (5.5 hours)
**Challenge:** build a shopping agent with an audit trail — reliable context, a meaningful action, and the steps behind it shown. Not a chat window.

Researched Nov 19, 2026. Each sponsor file has the same 12 sections (TL;DR, product, company, customers, why they're here, prize, what they want used, quickstart, past winners, fit ideas, risks, sources). Code in the sponsor files is copied or adapted from official docs and **has not been run**.

## Files

| File | What's in it |
|---|---|
| [00-event-brief.md](00-event-brief.md) | Organizer's text: rules, schedule, prizes, judges, Senso FAQ |
| [01-organizer-and-past-events.md](01-organizer-and-past-events.md) | Who runs it, past events and winners, patterns |
| [02-judges.md](02-judges.md) | Profiles of all 20 judges |
| [03-shopping-agent-landscape.md](03-shopping-agent-landscape.md) | Agentic commerce incumbents, what "audit trail" means, building blocks, project ideas |
| [sponsors/senso.md](sponsors/senso.md) | Senso — context layer (headline track) |
| [sponsors/superlinked.md](sponsors/superlinked.md) | Superlinked — SIE Serverless (two tracks) |
| [sponsors/amplitude-statsig.md](sponsors/amplitude-statsig.md) | Amplitude + Statsig — agent analytics, flags, experiments |
| [sponsors/guild-ai.md](sponsors/guild-ai.md) | Guild AI — agent control plane |
| [sponsors/airbyte.md](sponsors/airbyte.md) | Airbyte — context layer over live data sources |
| [sponsors/arcee-ai.md](sponsors/arcee-ai.md) | Arcee AI — open-weight models, "Pareto frontier" |
| [sponsors/cognee.md](sponsors/cognee.md) | Cognee — agent memory |
| [sponsors/firedrill.md](sponsors/firedrill.md) | Firedrill — agent simulation |
| [sponsors/singlestore.md](sponsors/singlestore.md) | SingleStore — real-time database |
| [sponsors/persona.md](sponsors/persona.md) | Persona — identity verification, Know Your Agent |

## Organizer, judging and what has won before

Full detail in [01-organizer-and-past-events.md](01-organizer-and-past-events.md) and [02-judges.md](02-judges.md).

- **Organizer:** tokens& (tokensand, LLC), formerly Creators Corner. Hosts are Alessandro Amenta, Jacopo Piazza and Marlene Ronstedt; venue is the AWS Builder Loft.
- **Senso is the organizer-aligned track.** Senso's founder Saroop Bharwani co-hosted this series until July, and the challenge statement mirrors Senso's pitch.
- **Judging criteria are not published for this event.** Every prior tokens& event used five criteria at 20% each: Idea, Technical implementation, Tool use (3+ sponsor tools), Presentation (3-minute demo), Autonomy.
- **What won before** (winners scraped for 21 of about 39 past events):
  - Overall winners usually also hold a sponsor track and use 3–4 sponsor tools deeply, not many shallowly.
  - They tackle real problems (health, compliance, safety) on live data, and open with a statistic.
  - Sep 25 overall winner: NurserAI, a permission-bounded family agent.
  - A Sep 25 track winner, Dead Reckoning, killed its agent mid-payment and showed no double charge next to a naive agent.
  - A shopping project that only returned links won nothing.
- **Judges worth knowing:**
  - Mogana Kumaran Sivaraman (Gap) is judging his fifth tokens& event; his public work is data-platform reliability, not storefront.
  - "Robert F." is Robert Farlow of Plato, which builds simulated environments that grade an agent's process.
  - Broccoli is Broccoli AI (home-services booking agents).
  - Lingxi Li is a design engineer; Riya Dashoriya's public record is accessibility.
- **Not verified:** winners of the October and Sep 12 events, any pre-built-code policy beyond "build during the event", Lingxi Li's current employer, and Harshal Vaidya's identity (medium confidence).

## Sponsor map

| Sponsor | Role it would play in a shopping agent | What wins the track (researcher's read) | Est. cost | Free access | Biggest caveat |
|---|---|---|---|---|---|
| **Senso** | Trusted retailer context: products, return and shipping policy | Senso as the gate before purchase: evidence ledger of `content_id` / `kb_node_id` / `version_id` / `chunk_text` / `score` / `rank` per decision; refuse when context is missing; live policy edit showing a new `version_id` | not estimated | API key from docs | Unknown whether evals are enabled on hackathon orgs |
| **Superlinked** | Small task-specific models as agent tools (rerank, extract, approve, redact) | Frontier model plans; 3–5 SIE models act; per-step ledger of model, latency, cost, evidence | not estimated | $20 credit, no card | Cold starts take minutes; 4 generation requests in flight per key |
| **Amplitude + Statsig** | Analytics and experiments over agent behaviour | Statsig assigns the agent variant, Amplitude AI SDK records sessions and tool calls, `Order Placed` funnel split by variant | 2–2.5 h, one person | Free tiers, no card | Scored sessions lag 15–20 min — seed early |
| **Guild AI** | Hosted runtime with policies and a per-session event log | Sub-agents (researcher, policy checker, buyer) with per-agent credential policies, approval before checkout, one visibly blocked call | not estimated | **Docs say no self-serve sign-up** | Account access must be confirmed |
| **Airbyte** | Live read-only context from Shopify, Gmail, Sheets, Stripe | Read context via the hosted MCP, log each call, place a test-mode order via `airbyte-agent-sdk` behind approval | not estimated | 14-day trial, no card | Unclear if trial orgs get the context layer; it cannot write |
| **Arcee AI** | The LLMs themselves, routed by cost | Cheapest adequate model per step, cost and latency logged, cost-vs-success plot from a 10–15 task eval | ~3 h | Credits for attendees not found | **"Pareto frontier" is not defined anywhere by Arcee** — ask the booth |
| **Cognee** | Shopper memory: preferences, purchase history | Cognee as the audit trail: evidence IDs from `recall`, session traces, one before/after correction | not estimated | $10 credit; sign-up may waitlist | Ingestion takes minutes; API verbs changed (`remember`/`recall`) |
| **Firedrill** | Test harness: simulated shoppers, synthetic Stripe, outages | Failing drill report → fix → passing re-run, shown in the audit UI | not estimated | 500 free + 500 with code `TOKENSAND` | CLI only, macOS/Linux, strict Node version; old example repos are stale |
| **SingleStore** | The one database: catalog search, orders, audit ledger, dashboard | Hybrid full-text + vector search, purchase and audit row in one transaction, live analytics | not estimated | Free Shared tier (10 tables, 1 GB) | Card requirement at signup unknown |
| **Persona** | Verify the human behind the agent | Verify human → link agent with spend limits → re-verify above threshold → inquiry ID in audit trail | 2–2.5 h | Sandbox, 60-day trial, no card | KYA is a framework, not an API; sandbox verification is simulated |

Time estimates are the researchers' own and only exist where shown.

## How the sponsors overlap

Three sponsors all call themselves a "context" or "memory" layer. Using more than one only makes sense if each holds different data:

- **Senso** — what the *retailer* says (catalog, policies). Versioned, so it is the natural source of citations.
- **Cognee** — what the agent knows about the *shopper* (preferences, past purchases, corrections).
- **Airbyte** — the shopper's *live accounts* (receipts in Gmail, a budget sheet, a Shopify store).

Similarly, four sponsors can each claim to be "the audit trail": Senso (evidence fields), SingleStore (ledger table), Guild (session event log), Amplitude (session view). One of them should be the system of record; the others feed or display it.

## Suggested track selection

This is my recommendation, not something the organizers published.

With 5.5 hours, a demo video to record, and 2–4 people, ten integrations is not feasible. Every sponsor's best angle is a variation of the same thing — *log the evidence for each step and show it* — so one well-built decision ledger can serve several tracks.

1. **Must have — Senso.** The challenge statement is Senso's track almost word for word.
2. **Cheap and structural — SingleStore.** If it is the only datastore, the track is satisfied by the architecture rather than by extra work.
3. **One of the "agent quality" tracks — Superlinked or Arcee.** Both reward per-step model, cost and latency logging. Superlinked has confirmed free credit; Arcee's track meaning is unconfirmed.
4. **One of the "trust" tracks — Persona or Firedrill.** Persona fits a spend-threshold story; Firedrill gives a strong fail-fix-pass demo beat.
5. **If a fourth person is free — Amplitude + Statsig**, which is self-contained work for one person.

Lower priority unless the booth clears the blocker: **Guild** (account access), **Airbyte** (trial access to the context layer), **Cognee** (ingestion latency and possible waitlist).

## Before kickoff

Accounts and installs are setup, not project code, so they can be done ahead:

- [ ] Cognee Cloud sign-up (may hit a waitlist)
- [ ] Senso account and API key
- [ ] SingleStore free Shared tier workspace
- [ ] SIE Serverless key (`pip install sie-sdk`)
- [ ] Amplitude project and Statsig project
- [ ] Persona sandbox (check whether a personal email is accepted)
- [ ] Firedrill: install `firedrill-run`, check the Node version requirement, redeem `TOKENSAND`
- [ ] Arcee API key
- [ ] Airbyte Cloud trial
- [ ] Guild: find out how teams get accounts

## Questions for the booths

| Booth | Ask |
|---|---|
| Arcee | What does "Pareto frontier" mean for judging? Are attendee credits available? Is `trinity-large-thinking` live on the API? |
| Guild | How do teams get an account? What are prize tokens redeemable for? |
| Airbyte | Does a trial org get the context layer? Is the hosted Agent Engine still supported? |
| Senso | Are evals enabled on hackathon orgs? What does a credit cost? |
| Superlinked | What separates the "harness" track from the "technical build" track? Any way to pre-warm models? |
| Cognee | Do session traces and the provenance ledger work on Cognee Cloud? |
| Persona | Are Relay and Signals enabled in a trial sandbox? |
| SingleStore | Do AI Functions work on the free tier? |
| Firedrill | Is the Web Corpus available to hackathon accounts? |
| Amplitude | Where is Agent Analytics enabled on a new project? |

## What could not be verified

- **Past winners** were found only for Senso (July 2026), Airbyte (eight winners across three 2026 events by this organizer), Superlinked (one London event), Cognee (one online event) and SingleStore (older university events). None were found for Firedrill, Guild, Persona, Arcee or Amplitude/Statsig.
- **No sponsor has published judging criteria** for its track at this event.
- Details per sponsor are listed in each file's "Risks and open questions" section.
