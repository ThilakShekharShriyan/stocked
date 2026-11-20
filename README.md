# Stocked

**The shopping agent for people who share a home.**

## The problem

When several people share a home, nobody owns the shopping. The dish soap runs out because everyone assumed someone else bought it. Two people buy toilet paper on the same day. A party or a visit from someone's parents changes what the house needs, but only one person knew about it. And when the group goes out, every cost depends on a number nobody can pin down: how many of us are actually going?

Keeping this straight takes constant messaging ("are we out of X?", "who paid?", "are you coming?") and someone doing arithmetic in a group chat. Tracking is friction, and so is communication.

## What Stocked does

Stocked is a shopping agent with an audit trail. Nobody tracks anything and nobody chats.

1. **It works out what to buy from context it can trust.** Purchase history comes from receipts already in the flat's inbox. Upcoming needs come from each person's calendar.
2. **It decides in code, not by guessing.** A model proposes a basket; code recomputes every price from the store catalog, picks online or pickup with an order-by time, and checks the house budget, the auto-approve limit, and each person's own limit.
3. **It takes the action.** Small restocks are ordered automatically. Larger ones wait for one tap from the approver, after an identity check. Group purchases (a dinner reservation, a cab) are booked for the real headcount and re-planned when someone drops out. All payments run in **test mode**.
4. **It shows why.** Every item, plan and order has a "Why" panel: the receipts it read, what was proposed, each check with its number, who approved, and which model ran each step and what it cost.

People only ever tap: *Approve*, *Skip*, *We're out*, *Plenty left*, *Can't make it*.

## How each sponsor tool is used

| Stage | Tool | What it does here | Where |
|---|---|---|---|
| Ingest | **Airbyte** (open source, PyAirbyte) | Syncs the flat's receipt inbox and every flatmate's calendar | `sidecar/airbyte_sync.py`, `core/src/ingest.rs` |
| Understand | **Superlinked SIE** | An open LLM drives the agent; a small extraction model reads receipt lines (each verified word-for-word); a decision model gates requests | `core/src/integrations/sie.rs`, `sidecar/sie.py` |
| Decide and record | **SingleStore** | The only database: purchase history, run-out estimates in SQL, plans, orders, outings and the append-only event log behind the audit trail | `db/schema.sql`, `core/src/db.rs`, `core/src/events.rs` |
| Ship safely | **Statsig** (Rust SDK) | A feature gate switches the planner between rules and the model; a dynamic config holds the auto-approve limit; every plan records its variant | `core/src/integrations/statsig.rs` |
| Measure | **Amplitude** Agent Analytics | Every conversation is sent as a session: user message, each tool call with input and output, the reply, and code-side scores, tagged with the variant | `core/src/integrations/amplitude.rs` |
| Improve | **Firedrill** | Simulated flatmates drive the agent against a synthetic Stripe; exact checks count the charges | `firedrill/` |
| Verify | **Persona** | The human behind an over-limit approval or booking completes an identity check; the inquiry goes in the audit trail | `core/src/integrations/persona.rs`, `core/src/verify.rs` |

## Architecture

```
web/      Next.js dashboard and phone view            :3000
core/     Rust (axum): the agent and every rule       :8080
sidecar/  Python: Airbyte sync, SIE extraction        :8090
db/       SingleStore schema
firedrill/ personas, evaluations, scenarios, world
data/     fictional inbox and calendars
```

The model proposes; Rust verifies. Checks, pricing and headcount arithmetic are pure functions with unit tests (`cargo test` in `core/`).

## Run it

```bash
cp .env.example .env.local   # fill in SingleStore; the rest is optional
./scripts/dev.sh             # core, sidecar and dashboard
```

Open http://localhost:3000, press **Sync inbox (Airbyte)**, then use the phone view on the right.

## What is real and what is simulated

- **Real:** the database, the Airbyte sync, the model calls, the checks, the Statsig evaluation, the Amplitude events, the Firedrill drills.
- **Simulated and labelled as such:** the flat, its receipts and calendars, the two stores and their prices, the restaurant and cab bookings, and all payments (test mode). The identity check is simulated until a Persona sandbox key and template are configured.
