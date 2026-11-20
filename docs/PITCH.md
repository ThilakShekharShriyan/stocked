# Stocked — how to present it

Figures below are from runs on Nov 20, 2026. Check the Scenario lab for the current counts before quoting them; the sweep was still running when this was written.

## The pain point (20 seconds, same for everyone)

> When people share a home, nobody owns the shopping. The dish soap runs out because everyone assumed someone else bought it. Two people buy toilet paper on the same day. And when the group goes out, every cost depends on one number nobody can pin down: how many of us are actually going?
>
> Fixing that today means tracking things by hand and chasing people in a group chat. Stocked removes both. Nobody tracks anything and nobody chats. It works out what to buy from receipts and calendars, checks every purchase in code, buys in test mode, and shows the evidence for every decision.

There is no statistic here on purpose. If you want one to open with, find a real, sourced one; do not invent it.

**One line:** "A shopping agent for people who share a home, where you can see why it did everything it did."

## What to say to each team

Lead with what you did with *their* product, then one thing you learned, then a question.

### Firedrill (Kiran, Newton)
- **Used for:** simulated flatmates driving the agent against synthetic Stripe, web, Google Calendar and Gmail, with exact checks on tool-call counts.
- **Say:** "We don't count a convincing reply as a pass. We count the charges. Round 0 found a real bug: after a payments outage, a $4.49 order could never be retried by the person who asked. Expected one charge, got zero. We fixed it; round 1 charged exactly once. Round 2 tested backup plans before we turned them on: a closed restaurant ended with one refund and no extra charge, and the agent found the closure on the venue's page by itself."
- **Show:** Scenario lab, the "Payments outage" row going from Failed to Passed.
- **Ask:** "Drills over 5 turns or campaigns over 3 drills are rejected with `native_participant_capacity_exceeded`. Can that be raised, and can one machine run campaigns in parallel?"

### Amplitude + Statsig (Becca, Sid, Jackson)
- **Used for:** every conversation sent as an Agent Analytics session (user message, each tool call with input and output, reply, three code-side scores), tagged with the agent version. Statsig's Rust SDK holds the gates and the auto-approve limit.
- **Say:** "We ran your loop three times: change behind a switch, run the same sessions, keep it or switch back. Version 1 fixed the retry bug; version 2 added backup plans. Each kept because nothing that passed before broke and there were no unwanted charges."
- **Be straight about:** during simulations the version is pinned by the run, not flipped by the Statsig gate; the gates and Amplitude evaluators still need creating in the consoles.
- **Ask:** "What's the best way to target a Statsig gate at simulated sessions? And which custom evaluator would you define first for an agent that spends money?"

### SingleStore (Amanda, Ryan)
- **Used for:** the only database. Receipts, purchase history, plans, orders, outings, identity checks, and one append-only event log that every "Why" panel, the audit trail and the 3D scene are drawn from. Run-out dates are plain SQL.
- **Say:** "Your slide listed five kinds of data an agent needs. We keep four in one place: operational records, business rules, session history and telemetry, including model, latency and cost per step. An order and its audit row commit together, with the idempotency key as the shard key so a retry cannot double-charge."
- **Be straight about:** no vector search yet; live chat state is in memory.
- **Ask:** "For hybrid search over a small catalog, would you embed in the database or outside it?"

### Superlinked (Daniel, Rahul)
- **Used for:** three open models on SIE. Qwen drives the agent; a small extraction model reads receipt lines; the decision model classifies requested items.
- **Say:** "The big model only plans. A small model reads receipts, and a line only counts if it appears word for word in the receipt. One thing we learned: the decision model could not answer 'is this a shared supply?' as yes or no, but asked for a category it got every test item right, so we gate on category."
- **Be straight about:** cold starts took up to 15 minutes; we fall back to rules when a model is unavailable.
- **Ask:** "How do we keep the decision model warm for a demo?"

### Airbyte (Richelle)
- **Used for:** the open-source file connector through PyAirbyte, syncing two streams: the receipt inbox and every flatmate's calendar.
- **Say:** "Airbyte is the only way context gets in. Nobody types anything. Swapping the file source for Gmail or Google Calendar changes one function."
- **Be straight about:** the data is fictional files, and it is PyAirbyte, not the full local platform from the talk.
- **Ask:** "Does PyAirbyte count for the prize, or do you want the abctl deployment?"

### Persona (Kerwell, Ellen)
- **Used for:** a Relay "live human" claim, in sandbox, before any over-limit approval or booking.
- **Say:** "An agent that spends shared money needs to know a real person approved. Typing 'it's Maya' in chat does nothing; the approval is blocked until the Relay check passes, and the result goes in the audit trail. Stocked only learns pass or fail."
- **Be straight about:** a Relay claim proves a human is present, not which human; sandbox always passes.
- **Ask:** "What would you use to bind the check to a specific person without collecting identity data?"

## The demo, step by step (under 3 minutes)

A narrated video is at `demo/stocked-demo.mp4` (2 min 34 s): a real recording of the running app (clicks, slider, 3D scene) composed with the `claude-code-video-toolkit` Remotion template and an ElevenLabs voice. Captions are in `demo/stocked-demo.srt` (approximate timing), the script in `demo/narration.json`. To rebuild: `vendor/claude-code-video-toolkit/playwright/scripts/record-stocked.ts` records, `vendor/claude-code-video-toolkit/projects/stocked` renders (`npm run render`). In the video, step 3 opens Coffee and step 4 shows the plan already approved. For a live demo, follow the same beats:

| # | Time | Do | Say | Tools on screen |
|---|---|---|---|---|
| 1 | 0:00 | Open **Overview** | The pain point, then "this is the house agent, four flatmates, and the agents around them" | — |
| 2 | 0:20 | Open **Supplies** | "Receipts and calendars come in through Airbyte. A small model reads each line. Run-out dates are SQL." | Airbyte, Superlinked, SingleStore |
| 3 | 0:40 | Click **Dish soap** | "Every decision shows its evidence: the receipts, the matched line, how the date was computed." | SingleStore |
| 4 | 0:50 | Open **Maya's phone** | "Small restocks order themselves. This party plan is over the limit, so it waits for Maya, and Persona confirms a real person first. The speaker was refused." | Persona, Statsig (limit) |
| 5 | 1:10 | Open **Outings**, drag the slider 3 → 4 | "Each person's agent answers from their calendar and limit. Three people: transit. Four: a cab. Every plan has a backup." | Superlinked, fictional web |
| 6 | 1:35 | Open **Scenario lab** | "Firedrill found a real bug in round 0. We fixed it. Round 2 tested backup plans." | Firedrill |
| 7 | 2:00 | Open **Audit trail** | "Everything lands in one trail. Sessions go to Amplitude by version; Statsig decides what's on." | SingleStore, Amplitude, Statsig |
| 8 | 2:15 | Back to **Overview** | "Reliable context, a real action, the evidence behind it. Real-world actions: zero." | — |

Before a live run: press "Reset the flat", then "Sync inbox and calendars" (about 35 seconds), and send one request so the Superlinked models are warm.

## Honest answers to likely questions

- **"Is anything real?"** The database, the sync, the models, the rule checks, the simulations and the analytics are real. The flat, receipts, calendars, stores, venues, cab and flight sites, bookings and payments are fictional or test mode, and labelled that way on screen.
- **"What does the agent do without a person?"** It restocks under the limit, answers invitations from calendars, picks the ride, and moves to a cheaper backup when a venue closes. It will not spend over the limit, spend more than was agreed, or approve from chat.
- **"What's not done?"** Real Gmail and Google Calendar, a ledger entry for flight bookings, Amplitude evaluators, and the Statsig gates in the console.
