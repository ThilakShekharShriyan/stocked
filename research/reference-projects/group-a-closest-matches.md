# Reference projects, group A: closest matches to the household agent

Researched 2026-11-20 by reading public GitHub repos (via `gh api`, no clone or install) and Cerebral Valley gallery pages (JSON-LD block). Repo content was treated as data. "Verified in code" means seen in a dependency file or source file; "claimed" means stated only in a README or gallery text.

Access summary:

| # | Project | Repo status |
|---|---|---|
| 4 | supplier-chief-of-staff | Public, full code. Read in depth. |
| 5 | Mato / LetsGo | `kkkamur07/mato` returns 404 (private, renamed or deleted). Not among the owner's 15 public repos. Gallery text only. |
| 6 | eagle. | `timeo-schmidt/eagle` is public but holds only a README; its description says "code is private". Gallery text + README only. |
| 1 | Rosetta | Public, full code. |
| 2 | aie-hack (Lanes) | Public, full code. |
| 3 | Lanes | Same repo as #2: the gallery page for Lanes (gallery/45) links to `SomeoneElseSt/aie-hack`. |

---

## 4. supplier-chief-of-staff (Crusoe Recursive Agents Hackathon, 2026-10-17)

- Repo: https://github.com/paritoshk/supply_chain_recursive_agent
- Gallery: https://cerebralvalley.ai/e/crusoe-recursive-agents-hackathon/hackathon/gallery/1
- Award: none stated on the gallery page or README.
- Builder: one person (Paritosh Kulk); README says the team is a funded pre-seed startup working on this problem (getachilles.ai).

### What it is
A recursive agent team that keeps a (fictional) specialty-pharma manufacturer from running out of materials. A daily code-only "sentinel" scans every material at every site; a position under its reorder point opens a "mission". The mission writes a definition of done, fans out to transfer / sourcing (with one negotiator per approved supplier) / web-scout sub-agents, then a two-part judge grades the plan. A human planner is pinged on Telegram only when a purchase is above $25k or checks failed.

This is almost exactly the household agent's "supplies run low, plan the purchase, ask the right person" loop, at industrial scale.

### Stack
Verified in code (`package.json` files, `.env.example`, `lib/models.ts`, `lib/store.ts`):
- Agent framework: `eve` (Vercel's agent framework, `eve ^0.80`), Vercel AI SDK (`ai`, `@ai-sdk/openai-compatible`, `@ai-sdk/openai`), Zod 4 for contracts. Monorepo with pnpm + Turborepo, TypeScript, Node 24.
- Models: Crusoe Managed Inference via OpenAI-compatible endpoint, one env var per role: planner/worker `NVIDIA-Nemotron-3.5-Super-VL`, verifier `Nemotron-3-Super-120B-A12B` (deliberately a different model from the doers), supplier persona simulator `Nemotron-3-Nano-30B-A3B` with thinking off.
- State: one JSON document in Upstash Redis REST (key `scos:state`), local JSON file fallback. Writers take a short Redis `SET NX PX` lock. Evidence-board items live in a separate Redis hash.
- Messaging: `eve/channels/telegram` (`telegramChannel`), approvals via `ctx.ask({display:"confirmation", options:[approve,reject]})` which renders inline buttons.
- Triggers: `eve/schedules` cron `0 13 * * *` plus an authenticated `POST /ops/sentinel` route (bearer `CRON_SECRET`) to fire it on demand during the demo.
- Web: Next.js 16, React 19, shadcn, Tailwind 4, three.js (a 3D "agent factory" view driven by the live event stream). Hosting: Vercel (`vercel.json`).
- Tests: Vitest; README claims 109 tests in core.

Claimed in README (not checked line by line): You.com search + MCP for supplier discovery, openFDA shortage/enforcement data, PubChem images on the evidence board.

### Architecture
- Agent loop (`tools/run_mission.ts`, an eve durable workflow tool): `prepare_mission` computes need qty and need-by date in code → writes six "done" checks (`covers_need`, `lands_in_time`, `qualified_only`, `comparable_quotes`, `batches_feasible`, `grounded`) → up to 3 attempts of: run 3 sub-agents in parallel → recompute everything in code (judge part 1) → a different model checks the doers' prose claims against those numbers (judge part 2) → if checks fail, turn each failed check into concrete feedback and re-plan. A loop guard stops if the same feedback repeats.
- Sentinel (`lib/sentinel.ts`): pure code, no model call. Skips positions that already have a mission in flight (and treats missions untouched for 20 min as abandoned), sorts worst cover first. Only a breach starts an LLM session, which keeps cost near zero on quiet days.
- Humans: `AUTHORITY_LIMIT_USD = 25_000`. Under the limit with all numeric checks passing and an approved supplier → auto-approve. Otherwise a Telegram message with the reasoning in five lines (cover, transfer, purchase, landed cost, failed checks) and Approve/Reject buttons. If no qualified source exists, it refuses to substitute and offers "Open qualification task / Not now".
- Verification: numbers are never taken from a model; quotes are only created from the supplier's last stored offer; "the plan the agent acts on is always the one recomputed in code". Judge grounding misses are reported but do not block a numerically sound plan.
- Learning (`lib/learning.ts`, `core/logic/learning.ts`): every failed check or judge-flagged claim becomes a stored "lesson" keyed by material and injected into the next mission's context. Deliveries update Beta-distribution beliefs per supplier (on-time, QC pass). Guardrails (approved suppliers only, no substitution, $25k) are explicitly outside what learning can change.
- Safety: `maxTokenCostUsdPerSession: 2` set on the agent; `FAULTS=persona_timeout,crusoe_429,bad_tool_json` env var injects failures for demo/testing.

### How results are presented
- A baseline-vs-agent table from a 90-day simulation over 107 positions, same demand and same injected delays for both (common random numbers): stockouts 7 → 2 (-71%), human minutes 21,450 → 228 (-99%), human touches 289 → 76, orders from non-qualified suppliers 0 → 0. Also a 10-seed mean.
- It openly states the trade-off (expedite spend rises sharply over 10 seeds) and labels the simulation synthetic, with the assumed per-task human minutes listed. This honesty makes the good numbers more believable.
- A "Real vs simulated" table, component by component.
- One concrete "hero" story: injectable sucrose at Ghent, 46 stockout days for the baseline vs 0 for the agent.
- A narrated live production run (6 of 6 checks passed, $6,128 auto-approved, ~3.5 min end to end) including a moment where the judge caught two misstated details.
- A learning table: rule-based vs risk-scored vs risk-scored + learning in a 360-day sim where 25% of suppliers lie about reliability (22% fewer stockout days).
- Live URLs: deployed agent, Telegram bot, "mission theater" web page with a simulated phone next to a 3D pipeline view, and an evidence board.

### Repo layout and size
16 commits between 18:41 and 02:58 UTC (about 8 hours). Roughly 850 KB of TypeScript (a large share is two web components: `theater.tsx` 114 KB and `agentic-factory-3d.tsx` 85 KB), ~16 KB Python (plot script). `apps/agent` (eve app: agent, tools, subagents, channels, schedules, lib), `packages/core` (types, inventory math, negotiation, seed generator, simulator, tools, tests), `apps/web` (Next.js demo), `evals/` (six benchmark suites B1–B6 with JSON results), `docs/` (12-sheet architecture PDF, sim results, model selection). README says the initial commit was empty and everything was built during the event.

### What the household team can borrow
1. **Code-only sentinel, LLM only on breach (SingleStore).** Run the "days of cover vs reorder point" calculation as a SQL query in SingleStore over the receipts-derived consumption table, on a schedule. Only rows that breach start an agent session. Store an `open_missions` row per supply so the scan does not re-alert while a plan is in flight, with a timeout for abandoned ones.
2. **Definition-of-done checks written before acting, recomputed in code, plus a different-model judge (SingleStore ledger + Senso).** For each shopping plan, write checks like `covers_need_until_event`, `within_budget`, `approved_store_only`, `grounded` as rows; recompute totals from the database, never from model text; have a second model check the explanation's claims. Store every check result in the ledger so "the evidence for every decision" is a query. If Senso is used, citations from the knowledge base can be what `grounded` checks against.
3. **Authority limit with one-tap approval message (messaging + Statsig).** Auto-approve under a threshold when all checks pass; otherwise send a short message (what is low, what will be bought, total, which check failed) with Approve/Reject. Make the threshold (and maybe who gets asked) a Statsig dynamic config or experiment, and log approve/reject/auto as Amplitude events to show "human touches" falling.
4. **Baseline-vs-agent simulation with the same random demand, and a "Real vs simulated" table (Firedrill + Amplitude).** Use Firedrill simulated housemates to run both a "manual" policy and the agent over N simulated weeks with identical consumption and events; report runouts, messages sent to humans, and spend. State the trade-offs and what is synthetic. Failed checks turned into stored "lessons" for the next run give a simple self-improvement story.

---

## 5. Mato / LetsGo (Crusoe Recursive Agents Hackathon, 2026-10-17)

- Repo: https://github.com/kkkamur07/mato — **not accessible: GitHub API returns 404.** The owner account exists with 15 public repos, none of them Mato/LetsGo. It may be private or deleted. Nothing below is verified in code.
- Gallery: https://cerebralvalley.ai/e/crusoe-recursive-agents-hackathon/hackathon/gallery/3
- Award: **Finalist** (from the gallery JSON-LD `award` field).
- Team: Krrish Agarwalla, Vibhuti Rajpurohit. Demo video: https://youtu.be/oji4MVncP9E (not watched).

### What it is (gallery text only)
Two agents:
- **LetsGo**: "An agent that lives in your chat. Ask it to plan, and it turns everyone's messages into rules." A group-chat planner that converts free-text preferences and constraints from several people into explicit rules before planning.
- **Mato**: learns from LetsGo's mistakes. It turns them into training and evaluation data, compares models, and fine-tunes a specialized Nemotron model, "keeping it only if it is better on unseen data". The page says they "heavily augmented data with synthetic data to train the model."

### Stack (claimed on gallery page only)
DSPy, FastAPI, React, SQLite, You.com search, Nemotron on Crusoe. Could not verify dependency files, models, or how the chat surface was connected (no repo).

### Architecture (inferred from the description only; treat as unverified)
- Agent loop: chat messages → rule extraction (likely a DSPy signature/module) → planning against those rules, with web search via You.com.
- State: SQLite (claimed).
- Humans: the group chat itself is the interface; each person's messages become rules.
- Verification / improvement: failure cases become a train/eval set; a fine-tuned model is promoted only if it beats the current one on held-out data (a promotion gate).

### How results are presented
Gallery text gives no numbers. The persuasive element is the two-agent story: an agent that does the job plus a second agent that improves the first from its own mistakes, with a held-out promotion gate.

### Repo layout and size
Unknown (repo not accessible).

### What the household team can borrow
1. **Messages → explicit, stored rules (SingleStore + Senso).** Parse each housemate's messages ("I'm vegan", "no shopping over $60 without asking me", "Sam pays for coffee") into structured rule rows with author, source message and timestamp. Store them in SingleStore; if Senso is used, keep them as versioned knowledge with citations to the original message so every plan can cite the rule it followed.
2. **Mistakes become eval data, with a promotion gate (Superlinked SIE + Statsig).** Log every rejected approval or corrected rule as a labelled example. If the team trains a small task-specific model (for example rule extraction or item categorisation from receipts), promote it only when it beats the current one on held-out examples, and roll the switch out behind a Statsig gate.
3. **Synthetic augmentation for a tiny dataset (Firedrill).** Use Firedrill simulated housemates to generate varied chat messages and edge cases to grow the eval set for rule extraction.

---

## 6. eagle. (Gemini 4 Argon SF Hackathon, 2026-10-25)

- Repo: https://github.com/timeo-schmidt/eagle — **public but contains only a 607-byte README.** The repo description says "code is private". Nothing below is verified in code.
- Gallery: https://cerebralvalley.ai/e/gemini-4-argon-sf/hackathon/gallery/75
- Award: **3rd Place** (gallery JSON-LD); the repo description adds "3rd place of 1,000+ applicants".
- Builder: Timeo Schmidt (one person listed). Demo: https://www.loom.com/share/ba5ba86becc542c68314095f9fb2ce0b (not watched).

### What it is (gallery + README)
VFR pre-flight planning for small-plane pilots from one sentence ("I want to fly to Monterey for dinner tonight with my wife Claudia"). It outputs a GO / NO-GO briefing; "you stay pilot in command". Then, in flight, a cloud agent watches conditions and talks to the pilot over a 160-character satellite text link.

### Stack (claimed only)
- Gemini 4 Argon for understanding the request and reading webcams and GOES satellite images; a **Gemini managed agent** in the cloud for in-flight monitoring (the event had a "Best use of Managed Agents" prize).
- A VFR autorouter on FAA data (airspace, terrain, obstacles, TFRs), NOTAMs.
- Risk via the FAA PAVE framework with hard no-go limits; fuel, performance, weight and balance "all calculated by code".
- A voice agent that calls the airport to confirm fuel, fees and a car.
- Messaging: satellite SMS-style text, 160 characters.
No dependency files to check.

### Architecture (from the description)
- Two phases. **Plan**: parallel checklist (route, weather, risk, logistics) producing several options (fastest, safest, most scenic) and a single GO / NO-GO summary. **Fly**: the approved plan is handed to a long-running background agent that keeps watching weather ahead of the aircraft, re-plans on change (Monterey goes IFR → divert to Watsonville) and sends only the decision in one short text.
- Humans: the pilot replies "Y" and "everything is re-planned" downstream (fuel, car, etc.). The pilot always makes the final call.
- Verification: hard limits and numeric calculations are done in code, not by the model; the model reads unstructured signals (webcams, satellite imagery) and explains.

### How results are presented
No metrics. Persuasion comes from one vivid, personal scenario (dinner in Monterey with a named person), a clear pain number ("every hour in the air takes about five hours of planning"), a memorable constraint and tagline ("My plane has 160 characters. The cloud has Gemini with a million tokens."), and a concrete re-plan moment in the demo (fog rolls in, divert, reply "Y").

### Repo layout and size
Public repo is README-only; the actual code size is unknown.

### What the household team can borrow
1. **Background re-planner that sends only the decision, answerable with one tap (messaging + Amplitude).** After a plan is approved, keep a watcher running (calendar changes via Airbyte, new receipts, price changes). When the plan breaks, send one short message such as "Party moved to Sat, 4 more guests. Add 2 pizzas + drinks ($38)? Reply Y". Track send → reply → time-to-reply in Amplitude.
2. **Hard limits in code, model only explains (SingleStore).** Keep budgets, allergy rules and "never buy X" limits as data plus code checks that produce GO / NO-GO; the model writes the summary. Store each GO / NO-GO with its inputs in the ledger.
3. **A tight message budget as a design constraint (Statsig).** Fix a short length limit for approval messages and test variants (one-line vs. itemised, with or without price) in a Statsig experiment, measuring approval rate and time-to-approve.
4. **One vivid named scenario for the demo.** Build the pitch around one concrete household event ("Maya's birthday Saturday, 12 people") with a live mid-demo change and a one-tap re-plan, rather than a feature list.

---

## 1. Rosetta (AI Engineer Code Summit Hackathon, 2026-11-07)

- Repo: https://github.com/KamayaniR/rosetta
- Award: README states "2nd Place, Best Use of Subquadratic", Software Archaeology track, solo build.

### What it is
Works out what an undocumented, minified JavaScript module does by running it, proposes rules about its behaviour, tries to disprove each rule with new experiments, ports the module to TypeScript, verifies the port against the original, and saves the verified behaviour as a test suite that blocks later edits that would change it. Demo target: `ms@2.1.3`, minified, with the original source hidden behind a path allowlist.

### Stack
Verified in code (`package.json`, `agents/requirements.txt`, `ui/package.json`, `src/llm.ts`, `src/store.ts`):
- Backend: TypeScript, Fastify, Zod, `fast-check` (property tests), Vitest, terser. Node `worker_threads` + `vm` sandbox.
- Models (`src/llm.ts`, one routing table): Meta `muse-spark-1.1` via OpenAI SDK pointed at `api.meta.ai`; Gemini `gemini-3.8-flash` via `@google/genai`; SubQ `subq-snap-1.1` (Decisions API, `api.subq.ai/v1`) for the Archivist.
- Database: MongoDB (`mongodb` driver) with collections such as `observations` (unique on session+run) and `events`; an in-memory fallback.
- Multi-agent: Fetch.ai `uagents` (Python) for six agents, with secp256k1 signatures (`@noble/curves`) checked by the backend.
- UI: React 18 + Vite + Monaco editor.
- Hosting: local only ("Runs locally; not deployed").

### Architecture
- Loop: Trace (sandboxed runs, each call 3x to catch nondeterminism) → Hypothesize (rules that cite real run IDs, each with an executable `applies`/`holds` check) → Experiment (3–8 new inputs per rule, at least one chosen to disprove it) → Archivist (long-context review of the whole history, reports untested branches and weak rules, feeding the next round) → Quirk Court → Port → Verify → Contract.
- State: every run, rule and event stored in MongoDB; the UI can replay a session from stored data.
- Humans: "Quirk Court": each surprising behaviour is shown to a person who marks Preserve, Fix or Flag (web UI or ASI:One chat). Applying decisions rebuilds and re-verifies.
- Verification: original vs port on all recorded inputs, 2,000 fuzz inputs and 500 held-out inputs never shown to agents; mutation test (3 planted bugs, all caught); every port line tagged with the rule it implements and coloured by evidence strength.

### How results are presented
A results table with hard counts (449 runs; 35 proven / 5 inferred / 16 disproven rules; 436/436 recorded, 2000/2000 fuzz, 500/500 held-out; 49/49 signed messages; 29 model calls split by provider). A list of surprising behaviours found blind ("`\"42 \"` parses but `\" 42\"` doesn't"). A "Checks" table where each safety claim has a script and a result. A "Sponsors" screen computed from stored data showing what each sponsor product did. An honest Limitations section.

### Repo layout and size
~330 KB TypeScript, ~36 KB Python; `src/` (~30 modules), `scripts/` (one script per check), `agents/` (Fetch.ai), `ui/`. 57 commits: 49 on Nov 7, 2 on Nov 8, 6 on Nov 17 (post-event polish).

### What the household team can borrow
1. **Every claim cites stored evidence IDs (SingleStore ledger).** Make each decision row reference the receipt IDs, calendar event IDs and message IDs it relied on, and show "supporting evidence" next to it in the UI, as Rosetta's rule ledger does.
2. **Try to disprove before acting.** Before trusting a prediction like "we use 1 carton of milk every 3 days", check it against held-out receipts; mark rules proven / inferred / disproven and only act on proven ones.
3. **A "Sponsors" panel computed from real data (Amplitude, Statsig, Airbyte, Senso).** Show judges what each sponsor tool actually did in the session (rows synced by Airbyte, events in Amplitude, experiment arm from Statsig, citations from Senso), computed from logs, not hand-written.
4. **A "Checks" table: one script per claim.** For each guardrail ("never orders over budget without approval", "never pays outside test mode"), have a script and show its result in the README.

---

## 2 and 3. aie-hack = Lanes (AI Engineer Code Summit Hackathon, 2026-11-07)

These are the same project: the Lanes gallery page (gallery/45) links to `SomeoneElseSt/aie-hack` and a Google Drive demo folder.

- Repo: https://github.com/SomeoneElseSt/aie-hack
- Gallery: https://cerebralvalley.ai/e/aie-code-2026-hackathon/hackathon/gallery/45
- Award: **2nd Place** (gallery JSON-LD). Team: Shehryar Hassan, Stiven Triana.

### What it is
An extension for the Pi coding agent that routes each user turn to a "lane" (a cheap or strong model, or a thinking-effort level) without breaking the provider's prompt cache. SubQ Snap classifies each turn as `one_shot`, `straightforward` or `complex`. Each lane keeps its own append-only thread so cached prefixes stay valid; lanes share facts through a scratchpad file rather than sharing context. It also distils a tiny local model for repetitive git requests.

Note: the gallery text and the README headline different numbers. Gallery: naive switching cost 40% more; easy turns 65% cheaper at 11.5/12 vs 12/12; hard tasks 2/5 and 0/7 before a fix, 5/5 and 7/7 after. README (updated after the event): effort routing saved about 18% on Gemini Pro ($0.681 vs $0.835) but did not beat always-medium; Meta routing was about break-even overall.

### Stack
Verified in code: TypeScript Pi extension (`extensions/lane-router/*.ts`); classifier calls SubQ `/systemone` (Decisions API) via `fetch`; `mappings.json` defines lane sets for Meta (`muse-spark-1.3-contributor`, `muse-spark-1.3`, `muse-wonder-1.0`) and Google (`gemini-flash-lite/flash/pro-latest`), plus effort-only sets. Pi and opencode are git submodules. Python scripts for evals (`evals/demo/run_demo.py`, `grade_demo.py`, `costsplit.py`) and distillation (`distill/mine.py`, `gen_data.py`, `evaluate.py`). A LoRA adapter file (`distill/models/v1/adapters.safetensors`, ~21 MB) is committed.
Claimed in README: LoRA training with `mlx_lm` on Qwen2.5-Coder-1.5B-Instruct 4-bit on Apple Silicon, served by `mlx_lm.server`.
No database: state is files (`.lanes/log.jsonl`, `.lanes/scratchpad.md`, `distill/roster.json`).

### Architecture
- Loop: classify turn in the background while the prompt renders → pick lane from mapping → the `context` hook gives the lane its own history → answering model may edit the scratchpad → next lane gets only the scratchpad delta as a hidden prefix.
- State: per-turn JSONL log of label, lane, model, tokens, cache read/write and cost; this log is "the accounting behind every number".
- Humans: the user just chats; no approval step.
- Verification: scripted 12-turn session replay graded by 5 objective checks plus 7 items an independent judge verifies; distilled models are adopted only if they pass held-out cases, stay within 5 points of the teacher and do not regress (`roster.json` records adopt/rollback decisions; a v2 at 88% was rolled back).

### How results are presented
Before/after cost tables with exact dollars per configuration, an explicit "It does not pay off everywhere" section, a caveats section (one scripted session, 1–3 runs, judges differ by run family), and a distillation table where the 1.5B student (100% held-out) beats its cloud teacher (65%) because it only trained on answers that passed execution. The candour about where it loses makes the wins credible.

### Repo layout and size
~112 KB Python, ~55 KB TypeScript (the extension itself is ~13 small files, ~50 KB). Large `docs/` research notes on Pi internals and a `gate/` folder with cached PR diffs for a merge-gate experiment. 20 commits: 8 on Nov 7, 7 on Nov 8, 5 after the event.

### What the household team can borrow
1. **Classify each request, route to a cheap or strong lane (Superlinked SIE + Statsig).** Use a small task-specific model to label incoming items (receipt line → category, message → "approval reply / new rule / chit-chat") and only send hard planning to the big model. Run "routed vs always-big" as a Statsig experiment.
2. **Per-turn cost and decision log as the source of every number (Amplitude + SingleStore).** Log each agent step with model, tokens, cost, label and outcome to SingleStore and as Amplitude events, so the demo's numbers are queries, not slides.
3. **Promotion gate with rollback history.** Keep a roster of model/prompt versions with adopt/rollback decisions and held-out scores; show one rolled-back version in the demo to prove the gate works.
4. **Shared facts, not shared context.** A small shared "household facts" store (who is home, budgets, rules) that each sub-agent reads as a delta keeps prompts short and cacheable; Senso could hold the versioned version of this.

---

## Cross-project summary: patterns most worth re-implementing

1. **Code-only trigger, LLM only on breach** (supplier-chief-of-staff): SQL in SingleStore computes days-of-cover; agent runs only for breaches.
2. **Definition-of-done checks recomputed in code + a different-model judge + failed checks become lessons** (supplier-chief-of-staff, Rosetta): numbers come from the database, never from model text; each check stored in the ledger.
3. **Authority threshold with one-tap approval message** (supplier-chief-of-staff, eagle.): auto under $X with all checks passing, otherwise a short message with Approve/Reject; threshold as a Statsig config; approvals tracked in Amplitude.
4. **Background re-planner that sends only the decision** (eagle.): watch calendars and receipts (Airbyte), re-plan on change, text "Reply Y".
5. **Messages → explicit stored rules with source citations** (Mato/LetsGo): rules table in SingleStore or versioned in Senso.
6. **Baseline-vs-agent simulation with identical randomness, plus "Real vs simulated" and "Checks" tables** (supplier-chief-of-staff, Rosetta): Firedrill housemates drive both policies.
7. **Promotion gate for small models with rollback** (Mato, Lanes): fits Superlinked SIE if used.

Could not access: Mato repo (404) and eagle. code (private; README only). Demo videos were not watched.
