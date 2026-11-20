# Stocked — UI handoff brief

You are picking up the front end of **Stocked**, a shopping agent for people who share a home. The backend works and is tested. Your job is to make the interface clearer and better looking **without changing what it says is true**. Read this whole file before editing.

State as of Nov 20, 2026, about 12:45 PT. Everything below was observed running unless marked otherwise.

---

## 1. What the product is

**Problem.** When several people share a home, nobody owns the shopping. Things run out, get bought twice, and every group cost depends on a headcount nobody can pin down. Keeping it straight takes constant messaging and someone doing arithmetic in a group chat.

**Product principle.** Nobody tracks anything and nobody chats. The agent infers what the flat has from receipts, plans from calendars, and people only ever *tap*: Approve, Skip, We're out, Plenty left, Can't make it, I'm in.

**Challenge it answers** (Personal Agents Hackathon): "Build a shopping agent with an audit trail" — use reliable context, take a meaningful action, show the information and steps behind it. Judging: Autonomy, Idea, Technical implementation, Tool use (sponsor tools), 3-minute demo. One judge is a design engineer and one has an accessibility background.

**The cast (fictional, seeded).** A flat at "14 Alder St": **Maya** (the approver), **Sam**, **Priya**, **Leo**. House budget $300 a month. Purchases under $25 are ordered automatically; anything over waits for Maya.

---

## 2. The two workflows

### A. Household supplies

1. **Sync** — receipts and each person's calendar arrive (Airbyte).
2. **Read** — a small model pulls line items from each receipt; a line only counts if it is found word-for-word in the receipt.
3. **Estimate** — run-out dates are computed in SQL from how often the flat re-buys an item. No quantities, just "runs out Thursday".
4. **Plan** — a calendar event (party, guests staying, dinner) or a low item triggers a proposed basket.
5. **Check** — code recomputes prices from the store catalog, picks online or in-store with an order-by time, and checks the monthly budget, the $25 limit, that every item is a shared consumable, and that nothing is already covered by another plan.
6. **Decide** — auto-order (under limit), hold for approval (over limit), refuse (not a shared supply, over budget, duplicate), or nothing to buy.
7. **Approve** — the approver must pass a Persona "live human" check, then taps Approve.
8. **Order** — one test-mode charge, cost split four ways, ledger updated. A repeat approve never charges twice.

### B. Outings (group purchases whose cost depends on headcount)

1. An outing appears on someone's calendar (a dinner, a day trip).
2. **Each person's agent answers for them** from two facts: their calendar (Leo is in Denver → out) and their personal spending limit.
3. The house agent prices it for the people going: reservation per head plus a ride. A cab is a fixed fare split by headcount and seats four; transit is per person. It picks the option that **keeps everyone within their own limit**, then the cheaper one, and says why.
4. **Headcount slider** — re-prices the outing for 1 to 6 people instantly and shows each person's cost against their limit.
5. The organiser books (after the same identity check). When someone drops out or joins later, the table size, the ride and everyone's share are re-planned in place.

---

## 3. Architecture

```
web/      Next.js 16 + React 19 + Tailwind 4      http://localhost:3000
core/     Rust (axum) — the agent and every rule  http://localhost:8080
sidecar/  Python (FastAPI) — Airbyte, SIE extract http://localhost:8090
SingleStore (cloud) — the only database
```

- The browser talks to **core** directly (`NEXT_PUBLIC_CORE_URL`, default `http://localhost:8080`, CORS open).
- Two Next.js route handlers exist only for Persona: `web/app/api/relay/session/route.ts` and `web/app/api/relay/redeem/route.ts`.
- Live updates: `GET /api/stream` is a server-sent event stream; the page refetches `/api/state` on each `change` event, with an 8-second poll as a safety net.
- Start everything with `./scripts/dev.sh`. All three services are running right now.

**Next.js warning.** `web/AGENTS.md` says this Next.js version has breaking changes and to read `web/node_modules/next/dist/docs/` before writing code. All current pages are client components (`"use client"`).

---

## 4. How each sponsor is integrated

| Sponsor | Role | What the UI shows today | Status |
|---|---|---|---|
| **SingleStore** | Only database: receipts, purchases, plans, orders, outings, verifications, and the append-only `agent_events` log | Everything; the Why drawer is that log | Working |
| **Airbyte** (open source, via PyAirbyte) | Syncs two streams from files: the receipt inbox and all calendars | "Sync inbox (Airbyte)" button; "synced by Airbyte" tags | Working. Data is fictional files, not real Gmail or Google Calendar |
| **Superlinked SIE** | Three models: an open LLM (Qwen) drives the agent and proposes baskets; `gliner2` extracts receipt lines; `GLiNER2.5-Decide` classifies a requested item into a category | Model name, latency and credits in the Why trail and the "Models used" panel; the category as a check | Working. The decision model can go cold and then falls back to a catalog flag |
| **Statsig** (Rust SDK, in core) | Gate `stocked_model_planner` switches rule planner vs model planner; config `stocked_house_rules` sets the auto-approve limit | Insights → "House rules" (limit, agent version, "live from Statsig") | Connected. Gate and config are **not created in the console yet**, so the agent runs as `v1-rules` with the default $25 |
| **Amplitude** | Every chat turn is sent as Agent Analytics events (user message, tool call with input and output, AI response, three scores) tagged with the Statsig variant; plan and order outcomes as product events | Nothing native yet beyond a funnel computed from our own data | Sending without errors; not checked inside Amplitude |
| **Firedrill** | Simulated flatmates drive `POST /chat` against a synthetic Stripe; exact checks count charges | Tests tab | First run done: 3 drills, all three charge counts correct. The UI shows only a "Raw report" link |
| **Persona** (Relay, sandbox) | "Live human" claim before an over-limit approval or booking | Persona's own widget inside the approval card | Server path tested end to end. The widget renders; nobody has completed it in the browser yet |
| Senso | Policy citations | Nothing | Not built |

---

## 5. The UI as it stands

Files (about 1,440 lines total):

| File | What it is |
|---|---|
| `web/app/page.tsx` | Dashboard shell: header, tabs (Home, Tests, Insights), demo controls, phone frame |
| `web/components/panels.tsx` | `RunningLow`, `ComingUp`, `Ledger`, `Panel`, `StatusPill` |
| `web/components/Outings.tsx` | Outing cards with the headcount slider |
| `web/components/WhyDrawer.tsx` | Right-hand drawer: the audit trail for an item, plan or outing |
| `web/components/PhoneView.tsx` | What one flatmate sees; includes `VerifyStep` and `RelayCheck` (Persona widget) |
| `web/components/tabs.tsx` | `TestsTab` (agent sessions, Firedrill reports), `InsightsTab` |
| `web/app/phone/[person]/page.tsx` | Standalone phone view, e.g. `/phone/maya` |
| `web/lib/api.ts` | Types, `api()` fetch helper, `useHouse()` hook, formatters, `covered()`, `verified()`, `eventSummary()` |

**Home layout.** Left: three columns — *Running low* (days-left bars), *Coming up* (calendar events with their plans), *Ledger* (who owes whom, decided plans). Below them: *Outings*. Below that: a dashed "Demo controls" row. Right: a phone frame with a Maya / Sam / Priya / Leo switcher.

**Visual language today.** Light stone background (`#f6f5f1`), white cards with `rounded-2xl`, teal for primary actions, amber for "waiting", red for "running out" or "refused", emerald for done, violet for TEST MODE and SIMULATED labels. Geist font.

---

## 6. Data contract

Everything on the page comes from `GET /api/state`. Types are in `web/lib/api.ts`; treat that file as the source of truth. Shape:

```
household  { id, name }
budget     { monthly_cents, spent_cents, remaining_cents, auto_limit_cents, agent_version, config_source }
members[]  { id, name, role: "approver" | "member" }
items[]    { id, name, status: low|soon|ok|cannot_estimate|event_only, days_left, runs_out_on,
             interval_days, basis, provenance: observed|inferred, correction, corrected_by }
calendar[] { id, title, starts_at, guests, kind: party|houseguests|dinner|away, nights, host_member_id, source }
plans[]    { id, title, kind, status: held|auto_approved|approved|ordered|refused|skipped|payment_failed|nothing_to_buy,
             total_cents, basket[]: { name, qty, unit_price_cents, store, channel, sourcing_note, why },
             verdict: { checks[]: { name, status: pass|fail|cannot_measure, detail }, reasons[] },
             agent_version, requested_by, trigger_ref, decided_by, decided_at, decision_note }
orders[]   { id, plan_id, total_cents, paid_by, splits[], test_mode }
balances[] / settlements[] { from, to, cents }
outings[]  { id, title, venue, starts_at, organizer, status: planned|booked,
             invitees[]: { member_id, name, status: going|declined, reason, by, cap_cents },
             quote: { headcount, ride: { mode, label, per_person_cents, extra_min }, alternatives[],
                      per_person_cents, total_cents, over_cap[], why },
             what_if[]: Quote for headcount 1..6, booking }
verifications[] { member_id, provider: persona|persona_relay|simulated, status, valid_until }
identity   { provider, sandbox, valid_minutes }
recent_events[] { kind, summary, actor, model, latency_us, cost_credits, provenance, data }
```

Other reads: `GET /api/why/{id}` (item, plan or outing id → item, plan, receipts with `evidence_span`, store options, events), `GET /api/sessions` (chat sessions with tool calls), `GET /api/firedrill/reports`, `GET /api/integrations`.

Actions (all `POST`, JSON body):

| Endpoint | Body | Used by |
|---|---|---|
| `/api/plans/{id}/approve`, `/skip` | `{ member, reason? }` | Approver's phone |
| `/api/items/{id}/correct` | `{ member, level: "out" \| "plenty" }` | Phone "still have some?" card |
| `/api/request` | `{ member, text }` | Phone "Ask the house" |
| `/api/outings/{id}/rsvp` | `{ member, status: "going" \| "declined" }` | Phone outing card |
| `/api/outings/{id}/book` | `{ member }` | Organiser's phone |
| `/api/verify/start`, `/api/verify/{id}/simulate` | `{ member, plan_id }` | Only when Persona is not connected |
| `/api/relay/session`, `/api/relay/redeem` (Next routes) | `{ member, plan_id }`, `{ verification_id }` | Persona widget |
| `/api/airbyte/sync`, `/api/invite/simulate`, `/api/scan`, `/api/reset` | `{}` | Demo controls |

Money is integer cents. Timestamps are naive UTC strings; `date()` in `api.ts` handles that. Errors come back as `{ "error": "..." }` with HTTP 400. An error starting `VERIFICATION_REQUIRED:` means the identity check is needed.

---

## 7. What to improve

Known weak spots, roughly in order of value:

1. **Tests tab.** The Firedrill card shows only "Raw report" because the summary extractor doesn't understand Firedrill's report format (`web/components/tabs.tsx`, `findResults`). The real results per drill are: exact check passed or failed, and rubric criteria (currently "not assessable"). The intro paragraph describes an old drill set. This tab should make "we simulated flatmates, here's what passed" obvious at a glance.
2. **Audit trail prominence.** The Why drawer is the answer to "audit trail" but is hidden behind small links. Consider making evidence visible earlier: sources → proposal → checks → decision → models and cost.
3. **Autonomy at a glance.** Nothing summarises "the agent did X on its own today". A small "what Stocked did" strip (ordered automatically, held, refused, re-planned) would help the Autonomy criterion.
4. **Outings.** The slider works but the per-person bars against limits are small, and the cab-vs-transit flip as you drag is the best moment in the product. Make it legible from the back of a room.
5. **Phone frame.** It is the main interaction surface in the demo but occupies a narrow right column. The Persona widget needs enough height inside the card (currently `min-h-64`).
6. **Insights tab.** The funnel is computed from local plans; "Models used" and "Connections" are plain lists.
7. **Demo controls.** A dashed row at the bottom of the page; fine functionally, visually an afterthought.
8. **Copy slips.** A day trip shows "Free that evening". "Sam's parents visiting · 2 guest(s) staying 3 night(s)" uses "(s)". Server-formatted times in `sourcing_note` are in UTC.
9. **Density.** The home page is long; Outings sit below the fold on a laptop.
10. **Loading and empty states** are minimal; sync takes about 35 seconds with only "Syncing…" on the button.

---

## 8. Constraints — do not break these

- **Honesty labels stay.** "TEST MODE", "SIMULATED", "sandbox", "simulated invite", "synced by Airbyte" and "seeded receipt" tell the viewer what is real. Restyle them; do not remove, hide or soften them.
- **Do not invent data.** Every number on screen must come from the API. No placeholder metrics, fake charts or hard-coded names.
- **Never colour alone.** Status always has text or an icon as well. Keep keyboard access, focus outlines, `aria-label`s on bars and icons, and tap targets of at least 44 px on the phone view.
- **The Persona widget is a human check.** Do not auto-click it, script it or bypass it. The Relay secret must stay in the Next route handlers; only `access_token` goes to the browser.
- **Do not change API shapes or backend rules** to suit a design. If something is missing from `/api/state`, note it rather than faking it.
- **Outing math is server-side.** Use `what_if[n-1]` for the slider; do not recompute prices in the browser.
- **No chat window as the main interface.** The brief is explicitly "beyond a chat window".
- **Secrets.** `.env.local` (and the `web/.env.local` symlink to it) holds live keys. Never print, commit or copy them.

---

## 9. How to check your work

```bash
cd web && pnpm exec tsc --noEmit        # must be clean
curl -s -X POST localhost:8080/api/reset
curl -s -X POST localhost:8080/api/airbyte/sync   # about 35 s; fills calendars, plans, outings
```

Then walk this script in the browser at http://localhost:3000:

1. Home shows 4 items running low, 4 calendar events from 4 calendars, a ledger, 2 outings.
2. Click an item → Why drawer shows receipts with the matched line highlighted.
3. Phone as **Sam** → tap "We're out" on an item → it moves and is marked "seen by a flatmate".
4. Phone as **Leo** → ask for "bluetooth speaker" → refused, with the reason, in Leo's card and the ledger.
5. Phone as **Sam** → "Can't make it" on Dinner at Luca's → the outing re-plans to 2.
6. Drag an outing's slider 1 → 6 → per-person cost and ride change; at 4 the dinner flips to a cab.
7. Phone as **Maya** → "Verify it's you" → the Persona widget appears. A person must complete it; then Approve appears and the order shows as TEST MODE in the ledger.
8. Tests tab lists agent sessions with tool calls; Insights shows house rules from Statsig.

Cross-check any figure against `curl -s localhost:8080/api/state`. The home screen matched the backend on every figure when last checked.

**Not verified by anyone yet:** completing the Persona widget through to Approve in the browser; booking an outing from the phone; the Insights tab after the latest changes; anything inside the Amplitude or Statsig consoles.
