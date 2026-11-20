# Stocked — session handoff

Written Nov 20, 2026, ~1:15 PM PT, for a new session with no memory of the previous one. Read this first, then `README.md`. For front-end work also read `docs/UI-HANDOFF.md`; for what to build next read `research/05-firedrill-eval-plan.md`.

## What this is

**Stocked** is a shopping agent for people who share a home, built for the Personal Agents Hackathon (San Francisco, Nov 20, 2026; submission deadline 4:30 PM PT; 3-minute demo). The owner has said at one point that they are not submitting, and at other points has worked toward the sponsor prizes. **Ask before assuming either.**

Challenge: "Build a shopping agent with an audit trail." Judging: Autonomy, Idea, Technical implementation, Tool use (sponsor tools), 3-minute demo.

It has two workflows: **household supplies** (receipts → run-out dates → basket → rules → auto-order, hold or refuse → one test-mode charge → cost split) and **outings** (each person's agent answers from their calendar and limit → headcount pricing → cab or transit → booking that re-plans when people drop out).

## Run it

```bash
./scripts/dev.sh        # core :8080, sidecar :8090, web :3000
```

Or separately: `cd core && cargo run --release`; `cd sidecar && uv run uvicorn app:app --port 8090`; `cd web && pnpm dev`.

- Dashboard: http://localhost:3000 · phone view: http://localhost:3000/phone/maya
- Clean demo state: `curl -X POST localhost:8080/api/reset` then `curl -X POST localhost:8080/api/airbyte/sync` (about 35 s).
- Checks: `cd core && cargo test` (20 pass) · `cd web && pnpm exec tsc --noEmit` · `cd web && pnpm build`.
- Toolchain on this Mac: Rust 1.88 (the `uuid` crate is pinned to 1.18.1 for it), Node 26.13.1 (upgraded for Firedrill), Python 3.12 through `uv`, pnpm.

## Layout

| Path | What |
|---|---|
| `core/` | Rust (axum). The agent and every rule. `plans.rs`, `checks.rs`, `outings.rs`, `verify.rs`, `ingest.rs`, `agent/chat.rs`, `agent/planner.rs`, `integrations/` |
| `sidecar/` | Python: Airbyte sync (`airbyte_sync.py`), SIE receipt extraction (`sie.py`). `statsig_cfg.py` is now unused; Statsig moved into the core |
| `web/` | Next.js 16, React 19, Tailwind 4, three.js. **Read `web/AGENTS.md`: this Next version has breaking changes** |
| `db/schema.sql` | SingleStore schema, applied at core start-up |
| `data/` | Fictional inbox and calendars read by Airbyte |
| `firedrill/` | Descriptor, personas, evaluations, scenarios, world, `collect_report.py`, `reports/` |
| `research/` | Sponsor research and the plans. `05-firedrill-eval-plan.md` is the current plan |
| `docs/` | This file and `UI-HANDOFF.md` |

## Secrets

All keys are in `.env.local` at the repo root (git-ignored). `web/.env.local` is a symlink to it. **Never print, commit or copy the values.** Names in use: `SUPERLINKED`, `SINGLESTORE` (management API key), `SINGLESTORE_HOST`, `SINGLESTORE_USER`, `SINGLESTORE_PASSWORD`, `SINGLESTORE_ADDR`, `AMPLITUDE`, `STATSIG_SDK_KEY`, `PERSONA_API_KEY`; `ARCEE` and `COGNEE` are present but unused. No Senso key.

Keys pasted into the old chat, which the owner may want to rotate after the event: the Statsig server key, the Persona sandbox key, and a Persona **production** key (never saved or used).

## Sponsor integrations: state

| Sponsor | State | Notes |
|---|---|---|
| SingleStore | Working | Only database. Dedicated S-00 workspace (not the free tier). Admin password was reset through the management API with the owner's approval |
| Airbyte | Working | Open-source `source-file` connector through PyAirbyte, two streams. Not the hosted context layer and not `abctl`. The connector needs its own Python 3.10 venv (`use_python="3.10"`) |
| Superlinked SIE | Working | Qwen 3.8 drives chat and planning; `gliner2-large-v1` reads receipts; `GLiNER2.5-Decide` classifies items |
| Statsig | Connected | Rust SDK in the core. Gate `stocked_model_planner` and config `stocked_house_rules` are **not created in the console**, so the agent runs as `v1-rules` with a $25 limit |
| Amplitude | Sending | HTTP API. Chat turns go as `[Agent] User Message` / `Tool Call` / `AI Response` / `Score`. Nobody has checked the Amplitude UI. `Session End` is not sent yet |
| Firedrill | Working | One campaign run: 3 drills, 3 of 3 charge checks passed. See limits below |
| Persona | Working (sandbox) | Relay "live human" claim. Server path tested end to end; the owner completed the widget at least once (a verified approval is in the event log) |
| Senso, Cognee, Arcee, Guild | Not used | |

## Things that will bite you

1. **SingleStore sleeps after 20 idle minutes** and takes about 3 minutes to wake. Resume it with the management API: `POST https://api.singlestore.com/v1/workspaces/8da2959a-c7c6-4b7e-bc81-ed57d191264b/resume` with the `SINGLESTORE` key as a bearer token. Core start-up fails with a connection error while it is asleep.
2. **This Mac's DNS will not resolve the database hostname**, so `SINGLESTORE_ADDR` holds its IP and the core uses that. If connections fail after a resume, re-check the IP with `dig +short $SINGLESTORE_HOST` and update `SINGLESTORE_ADDR`.
3. **Firedrill limits found by trial:** at most 5 turns per drill and 3 drills per campaign. More returns `native_participant_capacity_exceeded`. Rubric criteria came back "not assessable" because the evaluations ask for `actions` evidence; use `conversation` only.
4. **Superlinked models go cold.** A cold model answers `503 PROVISIONING`; the decision model took about 15 minutes to come up once. Their API also had two short outages today. The planner and the receipt reader have fallbacks; the chat agent does not.
5. **The decision model must be asked for a category, not yes/no.** As a yes/no question it could not tell dish soap from a speaker. `sie.classify` in `core/src/integrations/sie.rs` is the working form.
6. **After editing `web/app/globals.css`, restart `pnpm dev` and delete `web/.next`.** The dev server served a stale stylesheet once.
7. **The Persona widget is a human check.** Do not click it, script it, or call the redeem route to skip it. In sandbox the claim passes regardless, which is exactly why it must not be automated.
8. **Chat approvals are blocked by design.** Over-limit approvals need the identity check in the phone view; the chat agent returns a `VERIFICATION_REQUIRED` message. Firedrill scenarios expect 0 charges there.
9. **Statsig reads as `v1-rules`** until the gate exists, so event baskets come from the per-guest rule, not the model.
10. **Server-formatted times** in sourcing notes are UTC.

## Firedrill identifiers

Signed in through the browser on this Mac (the sign-in belongs to the owner's Firedrill account, not to any Claude account).

- Agent: `b7efd697-59f3-4fb9-8906-1695b309bc2b`
- Stripe tool version: `e07a00fd-9134-4a9d-b492-f30087e2ac58`
- Working 3-drill world: `2ab1cbda-c1f5-4b3f-8672-e817c90141db`; campaign run: `28fd1532-0f0c-4a62-ac24-7798219efc2f`
- Flow: `firedrill plan --world <id>` → `firedrill apply <plan> --approve --follow` → `firedrill run --plan firedrill/plan.json --descriptor firedrill/descriptor.json --campaign <id> --approve` → `python3 firedrill/collect_report.py <campaign> round-N`
- `firedrill/plan.json` is git-ignored and contains an absolute path to this repo.
- The fourth scenario (`restock_during_outage`) is saved but has never run.
- Credit balance is unknown; it was 2,000 before round 1. The owner should check the Firedrill app before more runs.

## What has and has not been verified

**Verified by running it:** the whole supplies flow and the whole outings flow through the API; dashboard figures match the backend; Persona server path; Firedrill round 1; Overview, Supplies, Outings and Scenario lab rendering in the browser after the redesign.

**Not verified:** Audit trail, Insights, the phone slide-over and narrow widths after the redesign; the WebGL fallback; anything inside the Amplitude or Statsig consoles; booking an outing from the phone; the outage Firedrill scenario.

## Git

The repo is initialised and files are staged, but **nothing has been committed**. The owner has not asked for a commit. `.env.local`, virtualenvs and caches are ignored.

## Next work

The owner asked for a plan before building: `research/05-firedrill-eval-plan.md`. It covers contingency planning (backup venue or ride, re-plan when something closes), more synthetic tools in the simulations (Google Calendar, Gmail, Web Corpus for taxi and flight sites, Stripe refunds), a scenario matrix of six suites, a five-layer evaluation design, and the Statsig → Firedrill → Amplitude "keep it or switch back" loop. **Nothing in it is built.** It ends with four decisions the owner has not yet made:

1. Flights and taxis as fictional sites in Firedrill's Web Corpus (recommended), or a custom Firedrill tool.
2. Build phases 1–4 today (recommended), or all six.
3. Accept that drill variants are set by an environment value, or get a Statsig console key so the gate itself is flipped.
4. The current Firedrill credit balance.

Smaller open items: Insights still has the old layout; the Supplies panels and phone slide-over are plainer than the rest; the pitch needs a real opening statistic (do not invent one); `docs/UI-HANDOFF.md` section 5 describes the UI before the redesign.

## Working agreements the owner has shown

- Wants detail planned before large builds, then fast execution.
- Wants sponsor tools used deeply and visibly, especially Firedrill, Amplitude + Statsig, SingleStore, Superlinked, Airbyte, Persona.
- Everything simulated must be labelled as such. No invented data, statistics or results.
- Ask before changing anything on their accounts (passwords, paid resources, console settings).

---

## Update, ~1:50 PM PT — phase 1 of the plan is done

The owner said "continue" on the plan with the recommended options (Web Corpus for flights and taxis, phases 1–4, variant set by an environment value during drills).

**Built and run:**

- `firedrill/suites.py` — the one tool for simulations. `python3 firedrill/suites.py save <A|B>` saves a suite; `python3 firedrill/suites.py run <suite> <round> <variant>` plans, applies, runs and collects. Suites: **A** supplies (small restock, speaker refusal, payments outage), **B** identity and limits (chat approval without a check, non-approver, splitting to dodge the limit).
- `AGENT_VARIANT` environment value pins the agent version for a run (set in `firedrill/plan.json` by `suites.py`). In the live app Statsig still decides.
- The core now shuts down gracefully: it records a `session_end` event for each open conversation (sent to Amplitude as `[Agent] Session End`) and flushes queued analytics.
- Evaluations use `conversation` evidence and the `instruction_adherence` dimension; some rubric criteria now score (others still come back "not assessable").
- **A real bug found by Firedrill and fixed.** Round 0 (agent v0), suite A: the payments-outage scenario ended with 0 of 1 charges, because retrying a small order that failed at the payment step went through the approver path. Fix (`plans::retry_payment`, active for every variant except `v0`): anyone can retry such an order. Round 1 (agent v1): 3 of 3, exactly one charge.
- Scenario lab is now a rounds × scenarios matrix with per-round metrics and a keep / switch-back decision computed from the reports. Round descriptions are in `firedrill/reports/_rounds.json`.
- Reports: `firedrill/reports/round-0-suite-a.json`, `round-1-suite-a.json`, and `round-1-suite-b.json` if the background run finished.

**Still to do from the plan:** phase 2 (refunds, calendar hold, `user_message` events, suite C), phase 3 (contingency planning behind a `stocked_contingency` gate, Web Corpus venue and taxi sites, suite D). Nothing from phases 2–6 exists yet.

**Credits:** each 3-drill campaign reserves up to 525 Firedrill credits; three or four campaigns have run. The balance is unknown; a run fails harmlessly at `apply` if credits run out.

---

## Update, ~2:10 PM PT — stopped mid phase 2 (usage limit)

**Finished and compiling, but never run:**
- Refunds: `Payments::refund` (`core/src/payments.rs`).
- A booked outing whose headcount changes now really charges the difference or refunds it, once per plan version (`core/src/outings.rs`, `rsvp`). If the payment step fails the booking is left unchanged.
- Chat agent has an `rsvp_outing` tool and lists outings in `get_house_status` (`core/src/agent/chat.rs`).
- With `DRILL_SEED=outing`, a simulation household starts with "Dinner at Luca's" already booked (`book_as_setup`).
- `firedrill/suites.py` defines **suite C** (drop-out refund, changes mind, questions only) with checks on both `payment_intents.create` and `refunds.create`.
- Firedrill descriptor revision 2 is registered (adds `refunds.create`); its digest is in `firedrill/descriptor.lock.json`, which `suites.py run` copies into `plan.json`.

**Exact next steps:**
1. Restart the main core so it runs the new binary: `pkill -f target/release/stocked-core; (cd core && ./target/release/stocked-core &)`.
2. Check suite B finished: `ls firedrill/reports/` should contain `round-1-suite-b.json`. If not, re-run `python3 firedrill/suites.py run B round-1 v1` (after step 3, because the descriptor changed).
3. Re-save every suite against descriptor revision 2: `cd firedrill && python3 suites.py save A && python3 suites.py save B && python3 suites.py save C`.
4. Run suite C: `python3 suites.py run C round-1 v1`. Expect failures on the first run; the Stripe `refunds.create` argument names are a guess (`payment_intent`, `amount`) and are unverified. Check with `firedrill tools catalog-operations bd159dd8-c741-5e60-a5b4-2e7319cd6ad5 --service stripe --page 3 --json`.
5. Run `cd core && cargo test` and `cd web && pnpm exec tsc --noEmit`; neither was run after these changes.

**Not started:** phase 3 (contingency planning, Web Corpus venue and taxi sites, suite D) and everything after it in `research/05-firedrill-eval-plan.md`.

---

## Update, ~2:15 PM PT — phases 2 and 3 run

Supersedes the "stopped mid phase 2" section above; its five steps are done.

- **Suite B, round 1 (v1):** 3 of 3, zero charges each.
- **Suite C, round 1 (v1):** 3 of 3 on first run. Charges and refunds matched exactly (drop-out: 1 refund; changes mind: 1 refund then 1 top-up charge; questions only: none). Charges are now created with `confirm: true` and the test card `pm_card_visa` so they can be refunded.
- **Contingency planning built** (`outings::venue_closed`, chat tool `report_venue_closed`, `POST /api/outings/{id}/venue-closed`). An outing's pricing may carry a `backup` venue. When the venue closes, the agent moves to the backup by itself only if it costs no more than agreed and keeps everyone in their limit, refunding the difference; otherwise it changes nothing and asks the organiser.
- **Gate:** `Sidecar::contingency` — in a drill, on for any `AGENT_VARIANT` other than `v0`/`v1`; in the live app the Statsig gate `stocked_contingency` (not created in the console, so it reads **off** live); on when Statsig is not connected.
- **Suite D** (venue closed with cheaper backup, backup costs more, question only; `DRILL_SEED=disruption` seeds two booked outings). Round 1 (v1, contingency off): 2 of 3 — the closed-venue scenario got 0 of 1 refunds. Round 2 (v2, on): 3 of 3.
- Reports: `round-0-suite-a`, `round-1-suite-{a,b,c,d}`, `round-2-suite-d`. `_rounds.json` describes rounds 0–2.
- Tests pass (20) and the dashboard type-checks. **Not looked at in a browser since these changes** (no browser tool in this session): the Scenario lab with the new rounds, and the "If this falls through" line and disruption banner on outing cards.

**Not built:** Google Calendar / Gmail / Web Corpus as synthetic dependencies (the agent still only calls Stripe outside itself), flight and taxi sites, suites E and F, repetitions, Amplitude custom evaluators, a UI button to report a closure. Firedrill credits: seven campaigns run; balance unknown.

---

## Update, ~2:35 PM PT — fake web, calendar, mail, cab and flights

- `core/src/world.rs`: the outside world. In a drill it calls Firedrill's synthetic Web Corpus, Google Calendar and Gmail (`WEB_INVOKE_URL`, `CALENDAR_INVOKE_URL`, `MAIL_INVOKE_URL`); otherwise a built-in stand-in (`simulated-local`). All venues, the cab company (swiftcab.test) and the airline (skyhop.test) are fictional.
- `core/src/agent/world_tools.rs` and chat tools: `check_venue` (reads the venue page; a closure notice triggers the backup path), `book_ride`, `check_calendars`, `find_flights`, `book_flight` (blocked over the limit unless identity-verified), `send_confirmation`. API: `/api/outings/{id}/check-venue|book-ride|send-confirmation`, `/api/world/venue`.
- Request shapes for the synthetic tools were learned from a Firedrill tool session (now stopped); they are encoded in `world.rs`.
- Second Firedrill agent `f337766c-ec36-4d19-a3bc-ef5350d6c3bc` (`descriptor.world.json`, four dependencies). **Suite E** uses it with `DRILL_SEED=world`.
- Suite E, round 2 (v2): closure found on the venue page → backup → 1 refund, 1 cab form submit, 1 email (all four checks passed on the second run; on the first run Firedrill's simulated person timed out and nothing was checked). Cab site down → no cab claimed. Flight for two → found, not booked, 1 calendar query.
- Outing cards have an "Outside world" row (check venue, arrange ride, email plan, demo closure notice). Scenario lab shows "not checked" for sessions Firedrill could not finish. **None of this UI has been seen in a browser.**
- Flight bookings charge but do not create a ledger order or cost split. `hh_wtest` is a leftover test household in the database.

## Update 3:45 PM — pitch, demo video, sweep

- **Pitch guide:** `docs/PITCH.md` (pain point, a talk track per sponsor team, live demo steps).
- **Demo video:** `demo/stocked-demo.mp4`. Recorded from the live app with Playwright (`vendor/claude-code-video-toolkit/playwright/scripts/record-stocked.ts`, pinned to playwright 1.48.2 because that browser was already installed), composed in Remotion (`vendor/claude-code-video-toolkit/projects/stocked`, `npx remotion render ProductDemo out/stocked-demo.mp4`). `vendor/` is git-ignored. The recorder's video clock runs ~9% slow; clips are re-timed with `setpts` (factor = recorded length / wall-clock length).
- **Voice:** ElevenLabs, key and voice ID read from `~/Projects/hawkeye/.env.local` (never print them). Script in `demo/narration.json`, clips in `demo/voice/`.
- **Deep links:** `/?section=Supplies|Outings|Scenario%20lab|Audit%20trail&phone=maya&select=<node>&why=<id>`; `&static=1` loads without the live connection (for screenshots, `demo/shoot.sh`).
- **Sponsor panels:** selecting a system in the 3D scene shows real usage figures (`useSystemFacts` in `web/components/overview.tsx`, totals from `/api/state`). Firedrill panel and Scenario lab link to https://app.firedrill.run/simulations.
- **Sweep (round 3):** `firedrill/sweep.py` (`queue`, `missing`, `collect`); campaigns must run one at a time. At 3:40 PM: 33 sessions, 31 passed, 2 not finished by Firedrill, 0 failed; S10 and S12 still queued.
- The user completed the Persona check themselves at 3:10 PM, so the party plan is approved in the demo flat.
