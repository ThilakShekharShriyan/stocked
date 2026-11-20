# Group B: Meta Global AI Developer Hackathon reference projects

Researched 2026-11-20 for the household-agent team (SingleStore, Airbyte, Amplitude + Statsig, Firedrill, maybe Superlinked SIE and Senso).

Method: each Cerebral Valley gallery page's JSON-LD block was read for the description, award and links. Each linked GitHub repo was read through `gh api` (README, dependency files, key source files). Nothing was cloned, installed or run, and no demo logins were used. Repo sizes come from the GitHub API (`size` in KB, and bytes per language).

Event: Meta Global AI Developer Hackathon, virtual, 26 Oct to 5 Nov 2026, two tracks (indie and startup), $1M prize pool. Sponsored inference partners: Baseten, Modal, Fireworks.

"Verified in code" means the dependency appears in a lockfile or manifest and is imported or called in source. "Claimed" means only the page or README says it.

Sections are written in the order they were researched: Punchwalk and Formwright first (the deep dives), then oqed, synthexis and Voluma Bio.

---

## 4. Punchwalk (gallery name "Consai")

- **Gallery:** https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/gallery/4672
- **Repo:** https://github.com/lucianobogo/punchwalk (public, MIT, created 2026-11-03, last push 2026-11-05)
- **Live:** https://punchwalk.fly.dev. Video: https://youtu.be/NBY_h07xsII
- **Award (stated in the gallery JSON-LD):** **Startup Track, 1st Place.** Web search: see the "Award check" section at the end.

### What it is
An inspector films a construction "punch walk" on Ray-Ban Meta glasses or a phone and says what is wrong. Punchwalk turns the video into a punch list (the defects to fix before close-out). Each item carries proof: the frame with the defect outlined, the exact second of video, the inspector's quote, and a written summary, fix, trade, room and severity. Verified items go to each trade's Slack automatically. Items the photo does not support wait for a person. On a re-walk it signs off fixed items and flags false "fixed" claims. The README says it was built in about 30 hours (first commit Nov 3, 3:06 PM ET).

### Stack: verified in code vs claimed
| Layer | Verified in code | Notes |
|---|---|---|
| Backend | FastAPI, uvicorn, SQLModel, pydantic, httpx (`pyproject.toml`) | Python 3.11, managed with uv |
| Models | `openai` SDK aimed at Baseten's OpenAI-compatible endpoint (Muse Spark Open, strict `json_schema` output); Meta Model API over httpx for Muse Voice Transcribe and SAM 3.1; `meta-sam-parser` | Muse Spark 1.3 is used only as a logged fallback when Baseten fails twice |
| Vision/media | opencv-headless, numpy, scipy, pillow, ffmpeg (Dockerfile) | ORB alignment of before/after frames, mask smoothing |
| Database | **SQLite in WAL mode** (`punchwalk/db.py`), one file on a Fly volume | JSON columns on `Item` for `verification`, `closure`, `history` |
| Messaging | **Slack incoming webhooks** (`dispatch/slack.py`, a plain POST of text plus blocks), email via Resend (env `RESEND_API_KEY`), and a "Copy link" fallback | Not a Slack bot: approval happens in the app, not inside Slack |
| Reports | Jinja2 and WeasyPrint for the PDF punch report, qrcode | |
| Frontend | React 19 PWA, Vite, Tailwind, TanStack Query, react-router, vaul; Playwright for screenshots | `web/` |
| Hosting | Fly.io (`fly.toml`): one machine, one volume, an in-process job worker | |
| Claimed only | Live glasses capture through Meta's Wearables Device Access Toolkit | Listed as future work |

### Architecture
- **Pipeline, not a free-form agent.** A fixed list of stages (`core/enums.py` `Stage`): ingest, transcribe, classify_turns, track_hands, find_points, fuse, pick_evidence, write_items, ground_defects, verify, resolve_closures, persist. The runner (`pipeline/runner.py`) makes every stage idempotent, cached, timed and resumable. Jobs cut short by a restart are re-queued on boot.
- **State:** SQLite tables for Project, Walk, Item, Dispatch, TradeContact and ModelCall. Each Item holds an append-only `history` list of events (actor = system or person, action, note, time), plus `verification` and `closure` JSON. Every external call is a `ModelCall` row with latency and estimated cost, plus a `cost_ledger.jsonl` file. All model answers are kept in a content-addressed cache (`fixtures/cache/`), so tests, evals and demo seeding replay at $0.
- **Verification (`pipeline/verify.py`):** a separate model call receives the rendered evidence photo, the item as written and the inspector's original words. It returns a typed `ItemCheck` with `consistent`, `frame_shows_defect`, `issues[]`, `suggested_fix` and `rationale`. If the check fails, or the call itself fails, the item becomes `needs_review` with a short plain note. Items are never dropped.
- **Human in the loop (`dispatch/policy.py`):** after processing, open items that passed the check are grouped by trade and sent over the trade's "effective channel": the trade's own Slack webhook, else email, else the team Slack, else Copy link. Every other item is held, and a history note says why ("Held for a look: ..."). A person releases it with "Approve and send to <company>". A send counts only if it reaches the trade. A channel failure stores a `Dispatch` row with `ok=false`, changes no item, and records a note ("Nothing was marked as sent"). Re-sends are idempotent, keyed on the closure's `decided_at`.
- **Re-walk sign-off (`pipeline/closures.py`):** each status update in the re-walk is matched to an open item, with a confidence threshold. Then the model looks at the before and after crops, aligned with ORB and the old outline redrawn. **A "fixed" answer is asked a second time, the other way round ("does it clearly show it fixed?"), and only signs off if both answers agree.** The decision table is explicit: claimed fixed and looks fixed → signed off; claimed fixed but still visible → needs a look (false-fix); claimed fixed but can't tell → needs a look; said not fixed → still open and sent again.
- **Guardrails:** a daily spend cap ($10) and a total cap ($60), uploads limited to 6 per hour per IP, a nightly reseed of the demo from a snapshot, and webhooks on example.com domains that are never contacted (so demo sends are safe).

### How results are presented
- The gallery and README lead with hard numbers on labelled walks: "12 of 12 defects found, 12 of 12 right trade and room, 2 of 2 false 'fixed' claims caught", "a 105-second walk becomes a punch list in under 5 minutes ... for about $0.54".
- An eval table lists each target set before tuning next to the result. It names a holdout walk that was never tuned on, and states the limits ("a careful smoke test, not a benchmark", "one miss would move recall 8 points").
- A blind before/after rating of v1 and v2 outlines: wrong outlines went from 6 to 0, at the cost of more items with no outline. Slogan: "no outline beats a wrong one".
- A cost table per walk built from the cost ledger, plus the whole build's spend ($3.69).
- A guided demo tour ("look at #003 ... marked fixed, still visible"), real phone screenshots, and a "How this walk was processed" screen that lists every model call.
- Why it persuades: every claim ties to an artifact a judge can open, the failure modes are named up front, and the product refuses to act when unsure.

### Repo layout and size
About 19 MB repo. Python ~1.6 MB (much of it is fixtures and scripts), TypeScript ~430 KB. About 936 files, most of them recorded-response cache. Folders: `punchwalk/{clients,pipeline,api,dispatch,render,report,core}` (~50 modules), `web/` (React PWA), `eval/` (labels, Hungarian matching, metrics, report), `tests/` (~45 test files, offline from fixtures), `scripts/` (seed, record, rehearsal, synth clip), `fixtures/`, `DESIGN.md`, `docs/DEPLOY.md`. The README credits an orchestrator agent plus worker coding agents, with each subphase verified before commit.

### What the household agent can borrow
1. **The send-or-hold gate with a reason on every hold → Statsig + SingleStore.** Auto-approve only when a separate typed check passes (for example "this purchase matches the receipt history and the event plan"). Otherwise hold the item with a one-line reason and send an "Approve and buy" request to the right housemate. Store `verification` and `hold_reason` on the decision row in SingleStore. Put the auto-send threshold, or auto-send on/off, behind a Statsig gate so you can show both arms.
2. **An append-only per-item history as the ledger → SingleStore (and Senso for cited policy).** Every action, by system or by a person, is an event row with actor, action, note and time. A "dispatched" event stores exactly what was sent. That is the "evidence for every decision" ledger. If house rules ("buy only unscented detergent") live in Senso, cite the version used in the event row.
3. **A double-asked confirmation before closing the loop → Firedrill.** Punchwalk signs off a fix only when two oppositely-worded checks agree. Re-implement it for "supply restocked?" or "order arrived?" and use Firedrill's simulated housemates to make false claims ("I already bought milk"). Report "N of N false claims caught", as Punchwalk did.
4. **A model-call ledger with costs, plus replay from cache → Amplitude.** Log each LLM or tool call as an event (stage, model, latency, cost, outcome) to Amplitude and to a SingleStore table. Seed the demo by replaying recorded answers at $0. Put a "how this decision was made" screen in the demo.

---

## 5. Formwright

- **Gallery:** https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/gallery/3836
- **Repo:** https://github.com/Romanok2805/formwright (public, PolyForm Noncommercial 1.0.0, created 2026-11-02, first commit cited as 27 Oct)
- **Live:** https://formwright.axo.kz (replay gallery at /replays). Video: https://www.youtube.com/watch?v=yIW7dZEq8gQ
- **Award (stated in the gallery JSON-LD):** **Startup Track, 2nd Place.** Web search: see "Award check".

### What it is
"Hardware R&D from a sentence." You describe a small device (a leak alarm, a plant sensor, a weather station). An agent on Meta's Muse models produces a schematic made of real LCSC parts, a placed and routed two-layer PCB, a 3D-printed snap-fit case and Arduino firmware, then prices a batch. You can edit any of it by hand in a node editor or board editor, and order it through the founder's existing 3D-printing business, Axo (Kazakhstan). The same agent also designs spare parts from photos.

### Stack: verified in code vs claimed
| Layer | Verified in code | Notes |
|---|---|---|
| Backend | FastAPI, uvicorn, pydantic-settings, httpx, `openai` SDK (`server/pyproject.toml`) | Python 3.12 |
| Models | `llm/client.py`: an `AsyncOpenAI` client on `meta_base_url` (Meta Model API, Responses API), a second client on `open_base_url` (Muse Spark Open on Baseten); direct httpx calls to `/asr/transcribe` and `/images/edits`; `meta-sam-parser` for SAM 3.1 | `infra/modal_glimmer.py` is an optional self-host on Modal. Muse Wonder, Muse Image and Spark 1.2 roles are claimed in the README; I did not trace each call |
| Database | **SQLite via aiosqlite** (`db.py`): about 19 tables, among them users, projects, programs (versions), jobs, events, messages, facts, memory, spend, orders, idempotency, replay_views, tg_chats | One replica, data on a Docker volume |
| Checks | `firmware_check.py` (~100 KB) and `firmware_sim.py` (~80 KB): the sketch compiles with `ziglang`, runs as WebAssembly under `wasmtime` against a DC model of the circuit; `device_rules.py` (~95 KB) design rules; a tscircuit autorouter and DRC in a sandboxed Node worker (`packages/pcb`) | Deterministic code, not the model |
| External APIs | JLCPCB/LCSC parts search and EasyEDA pinouts (live), OrcaSlicer CLI for print quotes, Axo partner API (`integrations/axo.py`), Telegram bot (`integrations/telegram.py`) | |
| Frontend | SvelteKit 2, Svelte 5, three.js, `@xyflow/svelte` (node editor), a local `@formwright/geom` package (manifold kernel) | |
| Hosting | Containerfile + Docker Compose + Caddy (automatic HTTPS) on one VPS; `deploy/smoke.sh` | |

### Architecture
- **Agent loop (`agent/loop.py`):** first a classify call with a strict JSON schema chooses a "pack" (device, pcb, enclosure, household ...), title, language and whether to refuse. Then a turn loop on the Responses API with an append-only history (for prompt-prefix caching). The tool list is trimmed per job (`tools_for`: "fewer choices, fewer tokens", in a stable order for caching). For the device pack the tools are search_parts, part_info, write_device, build_device, make_enclosure, export_fab, write_firmware, simulate_device, quote_device, look, inspect_board, inspect_case, inspect_part, grid, collage, dimensions, ask_user and finish. That is exactly 18, matching the page (counted from `tools_for`, `device_tools.TOOLS`, `device_inspect.TOOLS` and `pictures.TOOLS`).
- **Every tool result carries a "room" object:** `turn N of M` and `budget_left_usd`. Near the limit the loop injects "fix only blocking problems and call finish with an honest summary". At the hard limit a "last word" call must list what exists, what was not done and what was not checked. Reasoning effort is raised for the last turns.
- **"Done" is a gate, not a statement (`device_tools.finish_gate`):** when the model calls `finish`, plain code returns a list of reasons to keep going. The latest device version is not built; design-rule errors; requirement parts missing; case openings missing for a display, sensor or speaker; firmware fails its check; the virtual-board simulation fails; the board is oversized. The reasons go back to the model as the tool result. A model that stops talking without calling finish is nudged to call it, so the gate cannot be skipped. **After 3 blocked attempts (`BLOCK_ATTEMPTS = 3`) it ships anyway, and a handover message states the open problems.**
- **The simulation is made hard to fake:** each requirement can carry a scenario. A test that passes without its steps is refused, a test edited after failing is named in the result, and an unmodelled call reports "not simulated", never a pass.
- **Independent critic:** for printed parts, Muse Spark Open on Baseten sees only renders and photos, never the code. "The open model never decides a refusal: in the shop it can only hold an order for the operator."
- **State:** every write_device is a stored version (`fw.device/1`). The person's edits in the node editor come back as versions by "user", and the next agent run keeps them. Jobs, events (streamed to the browser over SSE), messages, spend and replays all live in SQLite. A finished run replays in the browser at no cost.
- **Memory (`memory/store.py`):** interface, pref, lesson and skill kinds, each with a hard budget. A lesson needs 3 pieces of evidence, a skill needs tests and 2 successful uses. Retrieval is top-k by SQLite FTS.
- **Ordering path (`integrations/axo.py`):** it POSTs the version that was quoted and checked to the Axo partner API (multipart: 3MF plus a JSON order) and gets back an order id, checkout URL, price and ETA, or a cart `add_url`. **An Idempotency-Key is stored in an `idempotency` table and passed to Axo**, so a retry returns the first order. An order with DRC errors, a changed schematic, an old case or no firmware is refused before it reaches the cart. Payment and board ordering are done by people (the shop's checkout, and a person at the workshop).
- **Human in the loop:** an `ask` tool lets the agent ask the person; there are direct-edit editors, an operator hold in the shop, and a Telegram bot.

### How results are presented
- One real run at the top of the README: "A desk weather station from a sentence, 4 min 1 s on Muse Spark 1.3", with a link to watch the replay and one screenshot per stage (schematic, board, case, firmware, price "$155.23 for five devices").
- The gallery tells a story of the checker catching the model: "64 tool calls, 4 min 25 s ... the checks caught a green LED where the brief said red, a transistor wired backwards and a buzzer that could never sound, and the model fixed each one."
- A public replay gallery of real runs. Judges can verify instead of trusting a video.
- An honest "What it does not do yet" section: the firmware has not run on real hardware, tests the model writes can be shallow, prices are estimates.
- A clear "built during the hackathon vs existed before" split (Axo existed before; only a small cart change was made during the event).

### Repo layout and size
About 8 MB repo, about 800 files. Python ~3.2 MB, TypeScript ~1.2 MB, Svelte ~860 KB, C++ ~220 KB (firmware simulator runtime and stubs). `server/formwright/` has ~378 files including `agent/` (loop ~55 KB, tools ~68 KB, device_tools ~85 KB, critic, narrate, prompts), `device.py` (~210 KB), `device_rules.py`, `firmware_check.py`, `firmware_sim.py`, `shop/` (intake, autoquote, QC, routes ~70 KB), `vision/`, `printing/`, `memory/`, `integrations/`, and 7 example devices. `server/tests/` has 144 files; `packages/geom` and `packages/pcb` are TypeScript packages; `web/` is SvelteKit. This is a very large build for one person in about 9 days.

### What the household agent can borrow
1. **A finish gate that returns reasons, with a cap of N blocks, then an honest handover → Firedrill + SingleStore.** When the agent says "plan done", deterministic checks run: the budget is not exceeded, every guest's dietary need is covered, nothing duplicates what is already in stock, each item has an approver, and the order total matches the quote. Failures go back to the model as a list. After 3 attempts, hand over to a person with the open problems listed. Use Firedrill's simulated housemates to create cases that should trip each check, and count "checks caught X" for the pitch.
2. **A budget and turn meter in every tool result → Amplitude + Statsig.** Return `turn k of N` and `budget_left` with each tool output, and change behaviour near the limit. Log turns, tool calls, blocked finishes and the final outcome per run to Amplitude. Run a Statsig experiment on the max-turns or effort setting and compare plan quality against cost.
3. **An idempotency-keyed order path that sends only the checked version → SingleStore.** The test-mode order sends the exact plan version that the housemate approved (store the version id and its hash), with an Idempotency-Key row in SingleStore. Refuse the order if the plan changed after approval, and say why.
4. **Gated memory → SingleStore (or Senso for the curated part).** Store housemate preferences and "lessons" (for example "Sam always rejects store-brand coffee") only after repeated evidence, with a budget per kind and top-k retrieval (SingleStore full-text or vector search). Senso could hold the curated, versioned house rules with citations. Also borrow **replays**: store the run's event stream so judges can play the agent's decision back.

---

## 1. oqed

- **Gallery:** https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/gallery/149
- **Repo:** https://github.com/IgnacioSearles/oqed (public, created 2026-10-26)
- **Live UI:** https://ignaciosearles.github.io/oqed (shows recorded runs). Video: https://youtu.be/IcxOSpjkJts
- **Award (stated in the gallery JSON-LD):** **Best Agentic Use-Case.** The README adds "one of 14 prizes across 5,367 projects" and links the prize announcement video (https://vimeo.com/1245287584#t=363.579; I did not watch it).

### What it is
An LLM-guided superoptimizer for x86-64. You point it at a C/C++ project. It builds and profiles the project, has Muse models write faster assembly for the hot functions, keeps only candidates that are correct and beat both `clang -O3` and `gcc -O3`, and ships them through a drop-in compiler wrapper (`CC=oqed-cc`) that re-proves each entry on every build and falls back to the compiler's output if anything changed.

### Stack: verified in code vs claimed
- **Verified (`pyproject.toml`, `src/oqed/providers.py`):** Python 3.12, `openai` SDK (`AsyncOpenAI`) aimed at `api.meta.ai/v1` with model ids `muse-spark-1.3` (volume) and `muse-wonder-1.0` (reasoning), plus `meta-models/muse-spark-open-1026` on `inference.baseten.co/v1`; each `ModelSpec` carries its per-token prices; angr (symbolic execution) + z3-solver for proofs; iced-x86 for instruction decoding (the ISA check); pyelftools; typer CLI; a Rust harness (`harness/`, a PyO3 0.30 extension) that runs candidates in a sandboxed child; llvm-mca; a React 19 + Vite + Tailwind UI.
- **Database:** none. State is an append-only JSONL event log per run (`runs/<id>.jsonl`, "the source of truth for the UI and for replay"), plus `oqed.lock` and `tricks.jsonl`.
- **Hosting:** the UI is a static GitHub Pages site that reads recorded runs.

### Architecture
- **Search loop (`search.py`):** many parallel candidates from the cheap tier, each pushed through filters: parse, duplicate, assemble, ISA, ABI, differential test, llvm-mca estimate, inline benchmark. **Every rejection goes back to the model with its specific reason** (assembler line, counterexample, ABI violation, crash registers, output diff). When progress stalls, the reasoning tier (Muse Wonder) gets the whole search history.
- **Verification:** an SMT proof when possible, and the label is honest: "proven" only when equivalence is decided, otherwise "tested, not proven". A suite of deliberately wrong rewrites (`tests/test_verify.py`) checks that the checker rejects them.
- **Typed events:** `RejectStage` is an enum (`parse | duplicate | assemble | isa | abi | test | verify | estimate | inline | benchmark`), so failure analysis comes free.
- **Learning without training (`tricks.py`):** each verified win becomes a "trick" (the C it replaced, the asm, a one-line explanation, its speedup). The most similar tricks (TF-IDF over tokens) go into later prompts. A function never sees its own trick, so the effect can be measured.
- **Spending cap per run** (`--max-usd`, default $1), with the spend printed.
- **Humans:** none in the loop. The human runs the CLI and reviews `oqed.lock`.

### How results are presented
A results table per real project: speedup vs clang and gcc, dollars spent, how many entries were proven. Honest failures are in the table too ("libyaml: no win, 99 candidates", "lz4: out of reach"). A "what we learned" section says LLMs are "decent at this, sometimes. not magic" and "the checker is the product", with a story of a zlib "win" that the checker caught making output 0.6% bigger. Cost per result ($15.85 for 1.17x on zlib). A live UI that replays every real run, including failure reasons and proofs.

### Repo layout and size
About 2 MB repo, 241 files. `src/oqed/` has ~27 modules (~310 KB: search 45 KB, project 48 KB, cc 34 KB, verify, oracle, inline, tricks), `harness/` (Rust), `benchmarks/` (30 functions), `sweep/` (per-project reports), `runs/` (~6.6 MB of event logs), `ui/` (React), `tests/` (13 files). Dense, mid-size.

### What the household agent can borrow
1. **A typed rejection-stage enum on every candidate decision → Amplitude.** For each proposed purchase, log `proposed`, then `rejected{stage: budget | duplicate_in_stock | approver_declined | out_of_stock | policy}` or `ordered`. Amplitude funnels then show where plans die, which makes a strong analytics slide.
2. **Feed the specific failure back to the model → Firedrill.** When a housemate (simulated in Firedrill) declines, or a check fails, return the exact reason ("Alex: over $40 for snacks") into the next planning turn rather than retrying blind.
3. **A JSONL/event log as the replay source → SingleStore.** Store one append-only events table per run and drive both the ledger UI and replay from it.
4. **A "tricks" library of verified wins in the prompt → SingleStore vector or full-text search (or Superlinked SIE for the similarity step).** Store approved past decisions ("Dec party: 2 bags ice per 6 guests, approved") and retrieve the nearest ones as examples. Hold out each event's own record so you can measure the lift.

---

## 2. synthexis

- **Gallery:** https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/gallery/5090 (gallery name "Team Synthexis")
- **Repo:** https://github.com/smashburger-dev/synthexis (public, created 2026-10-26, last push 2026-11-14, after the event)
- **Video:** https://youtu.be/EbNaVOHnnvo. No live site (it is a desktop plugin).
- **Award (stated in the gallery JSON-LD):** **Best Creative Build.**

### What it is
An FL Studio plugin that turns a webcam into a controller. You type a mapping in plain English or German ("when I open my left hand, open the filter"). It finds the right parameter in the chosen plugin, and from then on your hands, face or head drive that parameter live, recorded as FL automation. It ships readable parameter names for 887 plugins (107,350 labelled parameters). For unknown plugins, a small on-device model infers what each parameter does from its name. Everything runs locally on a Mac in about 100 ms.
The README also states it was built in the Indie track, Oct 26 to Nov 4.

### Stack: verified in code vs claimed
- **Verified (`plugin/synthexis/Cargo.toml` and workspace):** Rust with a vendored `nih-plug` (VST3/CLAP, standalone) and `nih_plug_egui` UI; `clap-wrapper` for the Audio Unit; `coremidi`; objc2 bindings to AVFoundation (camera), **Core ML** and **Apple Vision**; local crates `synthexis-parser` and `synthexis-complete`. Python training tools in `tools/` and `probes/`, and the README says training ran on Modal GPUs.
- **Meta models (claimed in the README, consistent with the training tooling):** none run at play time. SAM 3.1 produced pixel masks for 47,367 frames; SAM 3D Body filled 2,349 hard hands; Muse Spark Open labelled the parameters of ~1,290 plugins and wrote parser training sentences plus an 11,333-phrase glossary. From these came **5 small shipped models** (1.5M to 16M parameters, plus a 277 MB mmBERT-small parser) that run on the Neural Engine.
- **Database:** none. Shipped TSV/JSON label tables (`plugin/assistant/completion/known.tsv`).
- **Hosting:** GitHub Releases (a macOS installer).

### Architecture
Not an agent loop. It is an **offline distillation pipeline**: a big model labels data, and small task-specific models are trained on it and run locally. At runtime a sentence is parsed into "routes" (signal → parameter); the camera produces 57 signals per frame; the routes write host parameters through an FL Studio bridge script. State is profiles saved with the DAW project. Humans: the user types or edits routes. Verification: offline eval probes, each with a `RESULT.md`.

### How results are presented
- A "How well it works" table with latency (55 to 145 ms camera to parameter), accuracy on a held-out session (98.7% correct hand side), "ParamSense top-1 76.2% on 19 vendors it never saw" and "parser 65% exact".
- **A bar chart of parser versions v3 → v11 (15% → 63% exact), next to a direct baseline: "Muse Spark Open, asked to parse the same sentences directly, got 63%."** So the small model matched the large one at a fraction of the latency.
- Every number links to its source file or probe `RESULT.md`. Honest caveats: a holdout that was "partly optimistic", and a DAW support matrix that marks untested hosts.
- A large surface (887 plugins, 107,350 parameters, "6,118,950 single routes") makes the scope concrete.

### Repo layout and size
About 42 MB repo, 729 files. Rust ~2.1 MB, Python ~1.2 MB, Swift ~150 KB. `plugin/` (the synthexis crate ~1.8 MB of source across 50 files, the parser and completion crates, vendored nih-plug, ~9 MB of `models/`, ~36 MB of `assistant/` label data), `tools/` (mapping, labels, train, recorder, keypoints), `probes/` (327 files of experiments with RESULT.md), `spec/`, `docs/`. Large, research-heavy.

### What the household agent can borrow
1. **Distil a big model into a small task model → Superlinked SIE.** Use the big LLM offline to label a few thousand examples (receipt line → canonical item and category; "calendar event → expected supplies"), then use a small task-specific model at runtime. Show "small model vs big model on the same eval" with a version chart, as synthexis did.
2. **A probe folder where every number has a RESULT file → Senso (or the repo).** Keep each eval's method and result as a cited document. If Senso is used, the pitch's numbers can carry citations to versioned eval records.
3. **A plain-language rule turned into a structured route → Statsig.** "When we're down to one roll of paper towels, ask Priya" parses into a typed rule (signal, threshold, action, approver). Show it to the user before it takes effect. Statsig can gate an LLM parser against a small-model parser.

---

## 3. Voluma Bio

- **Gallery:** https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/gallery/4991
- **Repo:** https://github.com/voluma-bio/voluma (public, created 2026-10-26, last push 2026-11-05)
- **Live:** https://meta.voluma.bio. Model weights: https://huggingface.co/VolumaBio/mouse-knee-growth-plate. Video: https://youtu.be/qKdEp_oNrdA. (The page shows a judge login; it was not used.)
- **Award (stated in the gallery JSON-LD):** **Startup Track, 4th Place.**

### What it is
It turns a single micro-CT scan of a mouse knee into about 29 traceable 3D bone measurements for osteoarthritis research. Agents segment the bones, place regions and landmarks, measure and check their own work. A scientist reviews each result in a 3D and slice viewer beside its evidence and corrects it by typing. It is built for a specific lab (University of Rochester).

### Stack: verified in code vs claimed
- **Verified (package.json files across the pnpm/Nx monorepo):** TypeScript throughout. API on **Hono + Drizzle ORM + Postgres** (`postgres` driver), WorkOS AuthKit for auth, zod contracts, zarrita (Zarr volumes), sharp; a runner using the **Modal** JS SDK (sandboxes) and the AWS S3 SDK (object store); the agent harness on `@earendil-works/pi-agent-core` / `pi-ai` (a multi-provider LLM agent library); a gateway that proxies sandbox calls to model providers, "adds our key and checks a run's spend"; a React 19 web app with TanStack Query and a custom 3D viewer. Python workflow scripts (uv) for measuring and checking. Hosted on **Render** (`render.yaml`: api, gateway, runner worker).
- **Claimed in the README:** Muse Wonder as the main planner agent, SAM 3.1 prompted by agents, and a LoRA fine-tune of SAM 3.1 for the growth plate (weights on Hugging Face). I did not trace model ids in code.

### Architecture
- **A workflow is a set of skills:** `workflows/mouse-oa/` declares the outputs a run can produce (`outputs.yaml`), plus skills, each a written method with Python scripts that measure and check. A run snapshots the workflow so it can be reopened and re-checked later.
- **Agent loop (`packages/harness`, ~30 TS modules):** a main agent plans and starts up to 4 worker agents, each in its own Modal sandbox. Workers publish outputs **with the evidence they rest on** (files, the QC images they looked at, open questions). Modules include `approvals.ts`, `approval-jobs.ts`, `failure-policy.ts`, `context-overflow.ts`, `recorder.ts`, `supervise-run.ts`.
- **State:** an append-only **run event log** in Postgres (`RunEvent`), folded into the current outputs (`outputsFold`). Each output version records `madeFrom`, its inputs. **When a held call would create a new version, code walks the `madeFrom` graph and lists every downstream output it would make out of date** (`approvals.ts` `madeStale`). That list goes into the `approval.requested` event and is written by code, never by the agent.
- **Verification:** deterministic scripts refuse outputs that break the method (fused bones, swapped labels, impossible widths) and say how to fix them. When a step fails, main audits the failed work first and then redoes it. `failure-policy.ts` decides resume, replace or fail for a broken main agent.
- **Humans:** items the agents could not settle are marked "To check". The researcher sends a message with the current view attached, which starts a review agent that publishes a new version.

### How results are presented
"1 h 49 min, no human present, 29 measurements". "IIOC volume within 1.3% (2.949 vs 2.988 mm³) of a gold-standard manual tracing; femur diameter within 0.08 mm". A quote from the lab expert ("The image looks good"). The README opens with cited epidemiology (595M people, $80B) and the lab's own published method, so it reads as a real research tool, not a toy. The fine-tune result is stated against a ceiling: 16.4 µm from expert lines, where a perfect trace scores about 11 µm.

### Repo layout and size
The largest repo here: about 330 MB (mostly `.github/pr-assets/` screenshots, ~1,000 files), 1,688 files. TypeScript ~2.15 MB, Python ~680 KB. `apps/{web,api,runner,gateway,storybook}`, `packages/` (~15 packages: harness, contracts, viewer, run-view, provider-calls, object-store, ui ...), `workflows/` (core + mouse-oa skills), `training/plate-finetune`. A production-style two-person monorepo with AGENTS.md and CI.

### What the household agent can borrow
1. **Provenance on every ledger row, and approvals that list what they make stale → SingleStore.** Store `made_from` (receipt ids, calendar event ids, the stock estimate version) on each prediction, plan and order row. When a housemate edits the plan, compute in code which downstream items (the cart, the approval request) are now stale and show that in the approval message.
2. **Workflows as declared outputs plus skills with check scripts → Senso.** Declare what a run must produce (`restock_list`, `event_shopping_plan`, `approval_requests`, `order`) and keep the method text versioned in Senso (cited). Small check functions refuse outputs that break it.
3. **A gateway that holds keys and enforces per-run spend → Statsig/Amplitude.** Route every model call through one proxy that adds keys, meters cost per run and stops at the cap. Emit the cost per run to Amplitude, and use a Statsig dynamic config for the cap.
4. **"To check" plus correct-by-message → Airbyte inputs.** When Gmail receipts pulled by Airbyte cannot be matched to an item, mark that row "To check" and let a housemate correct it by replying. The correction becomes a new version of that row, not an overwrite.

---

## Award check

| Project | Award on the gallery page (JSON-LD `award`) | Other confirmation |
|---|---|---|
| oqed | Best Agentic Use-Case | The repo README says it won, "one of 14 prizes across 5,367 projects", and links the announcement video https://vimeo.com/1245287584#t=363.579 (not watched) |
| synthexis | Best Creative Build | The repo README says the same (Indie track) |
| Voluma Bio | Startup Track, 4th Place | Not found outside Cerebral Valley |
| Punchwalk (Consai) | Startup Track, 1st Place | Not found outside Cerebral Valley |
| Formwright | Startup Track, 2nd Place | Not found outside Cerebral Valley |

A brief web search (standard mode, 2026-11-20) found no independent press or winners list naming any of these five. The results covered other Meta hackathons (for example https://about.fb.com/news/2024/11/metas-llama-impact-hackathon-pioneering-ai-solutions-for-public-good/ and https://ai.meta.com/blog/llamacon-hackathon/). So the only sources for the awards are the Cerebral Valley gallery pages and the teams' own READMEs.

## Cross-project patterns for the household agent (summary)

1. **The checker is the product.** Every winner put deterministic checks after the model and made "the model doesn't get the last word" the headline (Formwright's finish gate, oqed's verifier, Voluma's check scripts, Punchwalk's verify stage). For the household agent: budget, stock, duplicate, approver and quote checks before any order. Firedrill supplies the adversarial housemates.
2. **Send-or-hold with a stated reason, and a person to approve.** Punchwalk auto-sends only verified items and holds the rest with a one-line reason. Voluma marks "To check". Map this to messaging the right housemate for approval, with Statsig gating the auto-approve threshold.
3. **An append-only event ledger with provenance, replayable.** All five keep an event log or versioned history (SQLite, Postgres, JSONL). In SingleStore: one events table plus decision rows with `made_from`, `verification`, `hold_reason` and the approver's response.
4. **Hard, honest numbers with a holdout, a before/after and the cost per run.** Punchwalk's "12/12, 2/2 false claims caught, $0.54", Formwright's "checks caught 3 bugs in 64 tool calls", synthexis's v3 → v11 chart against the big-model baseline. Plan a small labelled eval (simulated households in Firedrill) and an Amplitude dashboard of decisions, holds and cost.
5. **Spend caps and idempotent external actions.** Per-run and per-day caps (Punchwalk, Formwright, oqed, Voluma's gateway) and Idempotency-Keys on orders (Formwright).

## Access notes
- All five repos are public and were read through the GitHub API. Nothing was private or missing.
- No demo logins were used (Voluma's page shows judge credentials). Live sites were not opened. Demo videos were not watched.
- For Formwright and Voluma, some model roles (Muse Wonder, Muse Image, the SAM fine-tune) are taken from the README and were not traced call by call in code.
