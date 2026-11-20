# 01 — Organizer and past events

> Researched 2026-11-19. Everything here comes from public pages (tokens& site, Luma, Devpost). Items marked **inference** are my reading, not a published fact. Items marked **not found / unverified** could not be confirmed.

## 1. Who runs this

**tokens&** (legal entity "tokensand, LLC"), formerly branded **Creators Corner**. The Devpost organizer account for every one of their Devpost hackathons is still "Creators Corner" ([Devpost listing](https://devpost.com/hackathons?organization=Creators+Corner)), and older Luma pages show "Creators Corner" as presenter ([example](https://luma.com/vibecode)). Same hosts, same venue, same format.

| Item | Detail | Source |
|---|---|---|
| Event page | https://tokensand.com/personalagenthack | — |
| Luma | https://luma.com/personalagenthack (calendar: https://luma.com/tokensand) | Luma |
| Discord | https://bit.ly/discord-17 (used for team-finding) | event page |
| Socials | X [@tokensandai](https://x.com/tokensandai), [LinkedIn](https://www.linkedin.com/company/tokensand/), [Instagram](https://www.instagram.com/tokensand/), newsletter tokensandai.beehiiv.com | event page / Luma |
| Venue | AWS Builder Loft, San Francisco (address shown after approval; prior events list 525 Market St). Physical government photo ID required, 18+, approval-only | [Luma](https://luma.com/personalagenthack), [Devpost](https://loop-engineering-hackathon.devpost.com/) |
| Submission platform | tokens&'s own site: `/personalagenthack/submit`, gallery at `/personalagenthack/projects` (empty as of today). They moved off Devpost after the Jul 17, 2026 Loop Engineering hack | [gallery](https://tokensand.com/personalagenthack/projects) |

### People

Luma lists the hosts of this event as **tokens&, AWS Builder Loft, Alessandro Amenta, Jacopo Piazza, Marlene Ronstedt** ([Luma](https://luma.com/personalagenthack)).

- **Alessandro Amenta** — host on essentially every event since 2025; described as co-founder on the tokens& team page according to a search snippet (the page itself rendered no names when I fetched it, so treat the title as **unverified**) ([tokensand.ai/team](https://tokensand.ai/team)). A GitHub profile under the same name describes someone splitting time between San Francisco and Amsterdam who builds LLM projects ([GitHub](https://github.com/alessandroamenta)) — likely the same person, **unverified**.
- **Jacopo Piazza** — co-host on most 2026 events. No further public profile found.
- **Marlene Ronstedt** — co-host from Jul 2026 onward. No further public profile found.
- **Saroop Bharwani** — co-host on many events through Jul 2026 (Harness, Ship to Prod, Multimodal, Deep Agents, Autonomous Agents, Self-Evolving). He is **founder and CEO of Senso.ai** (YC W24) ([Crunchbase](https://crunchbase.com/person/saroop-bharwani), [YC](https://ycombinator.com/companies/senso)) and his X bio also references Zeta Global ([X post about a prior hackathon](https://x.com/saroopbharwani/status/1929572366277017666)). He is not listed as a host of this event.

**Why this matters:** Senso is not an ordinary sponsor. Its founder co-hosted this series for over a year, Senso has had a prize track at many past events, the challenge statement ("reliable context… audit trail") is Senso's product pitch almost verbatim, and the only FAQ the organizers published is Senso's. **Inference:** a strong Senso integration is the closest thing to "what the organizers want to see", and Senso's audit fields (`content_id`, `version_id`, `score`, `rank`) should be visible in the demo.

### Stated philosophy

- "tokens& produces high-signal technical events that drive product adoption, engineering hires, and founder deal flow." / "We get top engineers to build with your product." ([tokensand.ai](https://tokensand.ai/))
- Luma calendar tagline: "Give exceptional technical teams a path from invention to worldwide adoption." ([Luma](https://luma.com/tokensand))
- Model: curated monthly hackathons on a frontier theme; sponsors are the customers; "trusted by" logos include OpenAI, AWS, NVIDIA, Google DeepMind, Cursor, Anthropic, Vercel, ClickHouse ([tokensand.ai](https://tokensand.ai/)).
- tokensand.com itself is a product: free agent evaluations, project pages, build plans, perks ([tokensand.com](https://tokensand.com)). For this event they list themselves as an "optional builder tool" — compare the answer you expected with the one your agent gave before your demo ([event page](https://tokensand.com/personalagenthack)).
- For this event: "Personal agents can do far more than their demos show. Booking a flight or planning a trip? That's stuff you'd happily do yourself… most agents are still stuck inside chat apps… Think beyond the problems in SF." ([Luma](https://luma.com/personalagenthack))

**Inference:** the business is sponsor adoption. Sponsors pay to have engineers use their product, and sponsor staff make up most of the judging panel. Projects that make a sponsor's product look essential get rewarded.

Side note: a different, unrelated "Personal Agents Hackathon in SF" (hosts Lucia Viladrich and Gadi Borovich, Oct 24, ~50 engineers) focused on agents for everyday San Francisco tasks ([Luma](https://luma.com/0rycyhrt)). "Think beyond the problems in SF" may be a nod to it — **inference**.

## 2. Judging criteria and rules

### Criteria

Not published for this event (neither the event page nor Luma lists any). Every tokens&/Creators Corner hackathon from Oct 2025 through Jul 2026 used the same five equally weighted criteria, on Devpost and then on tokens&'s own platform:

| Criterion (20% each) | Wording |
|---|---|
| Idea | "Does the solution have the potential to solve a meaningful problem or demonstrate real-world value?" |
| Technical implementation | "How well was the solution implemented?" |
| Tool use | "Did the solution effectively use sponsor tools?" (some events: "at least 3 sponsor tools") |
| Presentation | "Demonstration of the solution in 3 minutes" |
| Autonomy | "How well does the agent act on real-time data without manual intervention?" |

Sources: [Loop Engineering Devpost](https://loop-engineering-hackathon.devpost.com/), [Self Improving Agents Devpost](https://self-improving-agents-hack.devpost.com/) ("at least 3 sponsor tools"), [Self-Evolving Agents builder brief on tokens&](https://tokensand.com/swarmhack). The Oct 9, 2026 Cyberdefense hack put it in the challenge statement: "Use 3+ sponsor tools" ([page](https://tokensand.com/cyberhack)).

2024–mid-2025 events used four criteria at 25% each (Idea, Technical implementation, Tool use — "creative use of at least 3 sponsor tools", Presentation) ([example](https://mcp-and-a2a-hackathon.devpost.com/)).

**Inference:** assume the five criteria apply tomorrow, and treat **3 sponsor tools as the floor**. "Autonomy" is the one teams forget: the agent should act on live data without someone clicking through each step.

### Rules

- Teams of 2–4, one project per team ([event page](https://tokensand.com/personalagenthack)). Past events capped at 4.
- "Build your project during the event." Past wording: "Projects must be built during the event." ([swarmhack brief](https://tokensand.com/swarmhack)). No published text about boilerplate, pre-written libraries or pre-made designs — **not found**. Safest reading: no project code before 11:00; accounts, API keys, sandbox stores and research are preparation, not project code. Ask on Discord if in doubt.
- Submit: GitHub repo reviewers can access, short demo video (past events: 3 minutes), description and tools used, team names and emails; website and screenshot optional.
- Flow: submission 4:30 PM → **finalist** demos and judging 5:00 PM → awards 7:00 PM. Past Devpost schedules show a judging round before finalist presentations (Loop: 4:45 judging, 6:00 finalist presentations). **Inference:** most teams are judged from the video and repo or a quick table pass; only finalists present live. The video and the project description carry more weight than the stage demo.
- Judging is by invitation; sponsors pick their own track winners (a "1st Prize Winner – General Judge" award exists separately on several events) ([Loop Devpost](https://loop-engineering-hackathon.devpost.com/)).

## 3. Previous hackathons

39 events are listed on [tokensand.ai/events](https://tokensand.ai/events). 23 have Devpost pages with winners; later ones use tokens&'s own gallery. All dates below are from those pages. Team names are as published on Devpost; tokens&'s own galleries mostly show "Anonymous builder".

### 3a. Most recent events (own platform — the best predictor of tomorrow)

| Event | Date | City | Sponsors | Projects | Winners |
|---|---|---|---|---|---|
| Cyberdefense Hackathon | Oct 9, 2026 | SF | ClickHouse, Pi, Akash, Guild.ai, Semgrep, **Senso.ai** | 78 | Not marked in gallery — **not found** ([gallery](https://tokensand.com/cyberhack/projects)) |
| Real-Time Video Agents Hack (SF / NYC / London) | Oct 2 / 9 / 17, 2026 | SF, NYC, LDN | VAST Data, NVIDIA, CoreWeave, Cursor | 45 (SF) | Not marked — **not found** ([SF gallery](https://tokensand.com/vastsf/projects)) |
| Long Horizon Agents Hackathon | Sep 25, 2026 | SF | AWS, Liquid AI, Nimble, Tinybird, Black Forest Labs | 55 | 11 winners, below ([gallery](https://tokensand.com/horizonagentshack/projects)) |
| Open Model Hack (Gradient x Google DeepMind) | Sep 12, 2026 | SF | Gradient, Google DeepMind | — | **not found** (no tokens& page) |
| Self-Evolving Agents Hackathon | Jul 24, 2026 | SF | Actian, Guild.ai, Replay.io, Band, **Senso**, Pioneer | 45 | Only one badge shown: **SwarmAds** — Best Use of Band ([gallery](https://tokensand.com/swarmhack/projects)) |

**Long Horizon Agents Hackathon winners** (same schedule and format as tomorrow; judges included Mogana Kumaran S. of Gap, who judges tomorrow):

| Award | Project | What it did | Links |
|---|---|---|---|
| **1st Place Overall** | NurserAI | Family assistant: local vision model turns camera frames into observations, long-term state store tracks caregivers, preferences and permissions, web search for unknowns; "only takes actions the family allows" and asks when it lacks permission. Can prepare a diaper reorder. | [project](https://tokensand.com/p/nurserai), [GitHub](https://github.com/zc275/LongHorizonAgentHackathon) |
| **2nd Place Overall** | Vigil | Re-checks ClinVar for patients with an uncertain genetic result and acts when a lab reclassifies it; every decision logged and queryable; extracted claims checked against page text "so a misread can't trigger a false alarm". | [project](https://tokensand.com/p/vigil), [GitHub](https://github.com/radShah/vus-watch) |
| 1st, Best use of Tinybird | Dead Reckoning | Kills an agent mid-booking right after payment, changes the site, restarts it with no memory; it recovers the receipt instead of paying twice. Append-only log and write-ahead protocol for every action. **Ran an "ordinary agent" beside it that pays twice.** | [project](https://tokensand.com/p/dead-reckoning), [GitHub](https://github.com/nihalnihalani/horizonhackathon) |
| 3rd, Best use of Tinybird | Livik | Agent that rewrites GPU kernels on a phone; measured +41% tokens/sec. | [project](https://tokensand.com/p/livik), [GitHub](https://github.com/a8nova/Livik) |
| 1st, Best use of Nimble | Local Ad Platform | Local-news-aware video ads; logs every input, intermediate and output. | [project](https://tokensand.com/p/local-ad-platform-lh), [video](https://youtu.be/lvvA5UkoleY) |
| Best use of Liquid AI | SiteLens | Construction hazard agent; every alert linked to an OSHA rule checked against osha.gov text; append-only evidence log. | [project](https://tokensand.com/p/sitelens), [site](https://sitelens-sim.vercel.app/) |
| BFL Flux Video / Image / Action | Local News Anchor / Long Horizon Wave / Riff | Generated news video; generated surf video; autonomous ad-testing loop with simulated audiences. | [1](https://tokensand.com/p/local-news-anchor), [2](https://tokensand.com/p/long-horizon-wave), [3](https://tokensand.com/p/riff) |
| 2 awards each (names not shown) | Verity, Breakout | Disaster-tracking pipeline; live AI news desk where "every step is written… as an event" and judges can query the log. | [Verity](https://tokensand.com/p/verity), [Breakout](https://tokensand.com/p/breakout), [video](https://www.youtube.com/watch?v=vEWdqSe6c6w) |

A shopping project at that event, **Seekr** (visual shopping from a 3D scene), did **not** win ([project](https://tokensand.com/p/seekr-see-find-remember)).

### 3b. 2026 Devpost-era events — overall winners

| Event (date, city) | Overall winner(s) | What it did | Links |
|---|---|---|---|
| Loop Engineering (Jul 17, SF; 193 registered, 65 projects) | **Fleet Yield Middleware** — 1st Prize, General Judge (Manas Sharma, Sanskriti Uma) | Reliability-aware GPU allocation that turns failure risk into placement decisions. 3 sponsor tools. | [Devpost](https://devpost.com/software/fleet-yield-middleware), [GitHub](https://github.com/manas15/risk-router) |
| Harness Engineering (Jun 12, SF; 78 projects) | **AeroRider** and **FDA SafetyNet** — Top 2 Overall | Bike routing scored on air quality and hazards, reroutes on its own; FDA recall feed → severity scoring → reaches affected patients. Both also won a sponsor track. | [AeroRider](https://devpost.com/software/aerorider), [FDA SafetyNet](https://devpost.com/software/silvershield-ofgwi2) |
| Agentic Engineering Hack (May 23, NYC; 75) | **TC-Pilot**, **RegRadar**, **Denial Rescue: RossMD** — Top Overall | Cancer-diagnosis navigator; credit-card compliance monitor (opens with "$2M fine. One query."); insurance-denial appeal packet with cited policy. | [TC-Pilot](https://devpost.com/software/tc-pilot), [RegRadar](https://devpost.com/software/regradar-0v3apw), [Denial Rescue](https://devpost.com/software/denial-rescue-rossmd) |
| Ship to Prod (Apr 24, SF; 61) | **HEARTH**, **ForgeRedemption** — Top Overall | Home-maintenance OS that predicts repairs and automates contracting; four agents in a prison-break game. | [HEARTH](https://devpost.com/software/hearth-39tbjx), [ForgeRedemption](https://devpost.com/software/forgeredemption) |
| Multimodal Frontier (Mar 28, SF; 71) | **Monitoring the situation**, **TrendPilot**, **interdimensional cable** — Top Overall | Live SF situational map from decoded emergency radio, aircraft and transit; trend-to-content agent; AI talk shows. | [Monitoring](https://devpost.com/software/monitoring-the-situation) |
| Deep Agents — RSAC (Mar 27, SF; 56) | **DataGrid** — Top Overall (+2 sponsor prizes) | Standardises messy multi-source patient records "with a full audit trail". | [Devpost](https://devpost.com/software/datagrid) |
| Autonomous Agents (Feb 27, SF; 104) | **RivalMap**, **Egg & Geese** — Top 3 Overall (third not marked) | One-URL competitor intelligence with graph memory that fires a battlecard without approval; agent swarm for social marketing. | [RivalMap](https://devpost.com/software/rivalmap), [Egg & Geese](https://devpost.com/software/egg-geese-8pzcvj) |
| Self Improving Agents (Feb 21, NYC; 68) | **CRUCIBLE** — 1st, General Judge (Alex Wu, Evan Correa) | Two agents play 100 rounds of Split or Steal; traces how deception emerges and distils it into defensive skill cards. | [Devpost](https://devpost.com/software/crucible-c3qabe), [GitHub](https://github.com/TheApexWu/crucible) |
| Continual Learning (Feb 6, SF; 52) | **Skills Cubed** — 1st, General Judge | MCP server that writes its own support playbooks from successful conversations. | [Devpost](https://devpost.com/software/skills-cubed) |
| Agentic Orchestration (Jan 16, SF; 73) | **Project Cerebro**, **Game Prototype Generator**, **AgentR** — Best Overall | Goal-to-achievement system; one prompt to game prototype; Amazon-seller agent. | [AgentR](https://devpost.com/software/agentr) |
| Multiagents Hackathon (Jun 26, London) | **not found** — Devpost page exists but was not in the organizer listing I scraped | — | [Devpost](https://multiagents-hackathon.devpost.com/) |

### 3c. 2024–2025 events — overall winners

| Event (date) | Overall winner(s) |
|---|---|
| Production Agents (Nov 21, 2025) | Urban, Briefly, ElderGuard AI ([gallery](https://production-agents-hackathon.devpost.com/project-gallery)) |
| Self-Evolving Agents Hack (Nov 15, 2025) | No overall award marked; sponsor tracks only |
| The Future of Agents (Oct 17, 2025) | CareSync, DROPS, PoliBerg, Breakdown, Guida AI |
| NYC AI Agents (Oct 4, 2025) | No overall marked; Vital Signal took 5 sponsor tracks |
| MCP – AI Agents (Sep 19, 2025) | PitchScoop (Overall Prize) |
| A2A – Agents (Jul 26, 2025) | STEVE, CharaLearn, NeuroTwin, MediaSKINoscope (Top 5) |
| MCP – AWS – Enterprise Agents (Jul 25, 2025) | Yc0n1c, Osirus.ai, LangBridge AI, Barter, STEVE (Top 5) |
| AWS MCP Agents (May 30, 2025) | SpotCheck, Find My Hospital, CareCopilot |
| MCP and A2A – AWS Edition (May 2, 2025) | Not marked in gallery — **not found** |
| AI Agents – GTC (Mar 22, 2025) | Lead Hunter, Know Your Founder |
| Multimodal AI Agents (Feb 15–16, 2025) | Billing Wizards (1st), HackNexus (2nd), ViVo (3rd), Metis AI (5th) |
| Vertical Specific (Oct 20, 2024), Voice & Video (Dec 7, 2024) | Winners not published on Devpost — **not found** |

Full per-sponsor winner lists are on each event's Devpost gallery, reachable from the [organizer listing](https://devpost.com/hackathons?organization=Creators+Corner).

### 3d. Past winners in tracks run by tomorrow's sponsors

| Sponsor | Event | Winner | Why it won (from the write-up) |
|---|---|---|---|
| Senso | Agentic Engineering Hack, May 2026 | **PolicyGuard** — "the compliance API for AI agents acting on the open web" | Agents ask "am I allowed to do this here?" before acting; used Senso plus x402/USDC payments ([Devpost](https://devpost.com/software/policyguard), [video](https://www.youtube.com/watch?v=Co7XsmoEV_k)) |
| Senso | Harness Engineering, Jun 2026 | **IncidentSherpa** | Postmortem "continuously emitted" from the agent's typed event log instead of reconstructed later; 8 sponsor tools ([Devpost](https://devpost.com/software/incidentsherpa)) |
| Senso | AWS MCP Agents, May 2025 | Novelty Research | Real-time research breakthroughs feed |
| Senso | MCP and A2A, May 2025 | Sidekick | AI workspace for PMs |
| Guild.ai | Harness Engineering, Jun 2026 | **DailyGate**, proPR, Magpie | DailyGate: agent that "earns its autonomy" — starts by asking approval, widens what it does alone based on what you approved ([Devpost](https://devpost.com/software/dailygate)) |
| Guild.ai | Ship to Prod, Apr 2026 | Phalanx, Branch, WildFire Response, tracepath, **RxScout**, MediCall | RxScout: parallel browser agents compare cash price of a prescription across pharmacies ([Devpost](https://devpost.com/software/rxscout)) |
| Airbyte | Harness Engineering, Jun 2026 | Hestia, FDA SafetyNet, SayHello! | "Conquer with Context" track |
| Airbyte | Deep Agents, Mar 2026 | Sift, Deep Sentinel, Signal | Agents that connect several real tools and act on what they find |
| Airbyte | Autonomous Agents, Feb 2026 | backoffice.ai, RivalMap | — |

### 3e. Shopping and commerce projects at past events (direct precedents)

| Project | Event | Result | Note |
|---|---|---|---|
| BasketIQ | Agentic Engineering Hack, May 2026 | Won Nimble track | Grocery receipts → what to buy where; store-by-store recommendation ([Devpost](https://devpost.com/software/basketiq)) |
| RxScout | Ship to Prod, Apr 2026 | Won Guild.ai track | Price comparison across pharmacies with browser agents |
| Agent Negotiation platform | Harness Engineering, Jun 2026 | Won Pioneer track | Buyer and vendor agents negotiate then pay through Stripe ([Devpost](https://devpost.com/software/agent-negotitation-platform)) |
| EARWITNESS | Harness Engineering, Jun 2026 | Won ClickHouse track | "The agent that pays for what actually aired" — verifies the signal, then moves real USDC ([Devpost](https://devpost.com/software/earwitness-the-agent-that-pays-for-what-actually-aired)) |
| AutoRescue | Future of Agents, Oct 2025 | Won a sponsor award | Post-purchase rescue: detects delayed shipment, offers reship/credit/refund, executes ([Devpost](https://devpost.com/software/autorescue-agentic-post-purchase-rescue)) |
| ProdSpecGuard | Agentic Orchestration, Jan 2026 | Won Retool track | Monitors e-commerce listings for spec errors |
| eBaySnipe | AWS MCP Agents, May 2025 | Won MiniMax track | Monitors eBay for new listings and acts |
| Personal Shopper Agent | AI Agents – GTC, Mar 2025 | Won Apify track | Shop from a preferred set of stores |
| Seekr | Long Horizon, Sep 2026 | No award | Visual product discovery; ends in a list of links |

**Inference:** the shopping projects that won did something measurable (price difference, money moved, order rescued). The one that did not win stopped at "here are some links".

## 4. Patterns

1. **Sponsor-track wins outnumber overall wins roughly 10 to 1, and overall winners usually also hold a sponsor track.** Of the 2026 overall winners above, AeroRider, FDA SafetyNet, TC-Pilot, HEARTH, ForgeRedemption, DataGrid, RivalMap, AgentR and Monitoring the situation each also took at least one sponsor prize. Design for two or three specific tracks, not all ten.
2. **Sponsor-tool count:** winners on the own-platform events listed 3–4 sponsor tools each (every Long Horizon winner used 3 or 4 of the 4 available). Published floor is 3. Some track winners used 8, but single-purpose winners with 3 deep integrations are just as common. Depth is described in the write-up tool by tool ("X is its only memory", "Y reads the status page").
3. **Real, slightly unglamorous problems with a number attached.** Healthcare, compliance, safety, elder care and local government recur among overall winners. Opening lines carry a statistic: "$2M fine. One query.", "80.7% of appealed denials were overturned", "One cholesterol pill: $7 at Costco, $393 across the street."
4. **Live or real data, not fixtures.** openFDA, ClinVar, osha.gov, GH Archive, live radio, real ferry schedules. This maps to the Autonomy criterion ("acts on real-time data").
5. **Evidence and logs as a feature, well before this theme.** Append-only logs, cited sources, "every decision logged and queryable", claims checked against source text. Tomorrow's theme makes that the headline.
6. **Permissioned action.** NurserAI (1st overall, Sep 2026) and DailyGate both centre on what the agent is allowed to do alone and when it must ask.
7. **Demo style:** a 3-minute video, usually YouTube or Loom; one concrete scenario walked end to end; a failure injected on purpose (Dead Reckoning's SIGKILL) or a side-by-side with a naive agent; a dashboard rather than a chat window. Several winners deployed a public URL.
8. **Team size:** solo builders and pairs win often (RivalMap, AeroRider, FDA SafetyNet, IncidentSherpa, EARWITNESS, CRUCIBLE were 1–2 people). Scope discipline beats headcount.
9. **Scale:** 45–105 submitted projects per event, 130–210 registrants. Expect 50–80 competing projects tomorrow.

## 5. Could not verify

- Judging criteria for this specific event (not published).
- A detailed pre-built-code policy beyond "build your project during the event".
- Winners of the Cyberdefense (Oct 9), Real-Time Video (Oct 2/9/17) and Open Model (Sep 12) hacks, and the London Multiagents hack.
- Overall winner of the Jul 24 Self-Evolving Agents hack (only a Band-track badge is shown).
- Team names for own-platform winners (shown as "Anonymous builder").
- Organizer recap posts on LinkedIn or X naming winners (LinkedIn and X were not readable without login).
- Titles and backgrounds of Jacopo Piazza and Marlene Ronstedt; Alessandro Amenta's "co-founder" title.

## Sources

- https://tokensand.com/personalagenthack · https://tokensand.com/personalagenthack/projects · https://tokensand.com
- https://tokensand.ai/ · https://tokensand.ai/events · https://tokensand.ai/team
- https://luma.com/personalagenthack · https://luma.com/tokensand · https://luma.com/horizonagentshack · https://luma.com/swarmhack · https://luma.com/cyberhack · https://luma.com/loophack · https://luma.com/harnesshack · https://luma.com/vastsf · https://luma.com/shiptoprod · https://luma.com/multimodalhack · https://luma.com/deepagentshack · https://luma.com/sfagents · https://luma.com/agentslab · https://luma.com/openmodelhack · https://luma.com/vibecode · https://luma.com/0rycyhrt
- https://tokensand.com/horizonagentshack · https://tokensand.com/horizonagentshack/projects · https://tokensand.com/swarmhack · https://tokensand.com/swarmhack/projects · https://tokensand.com/cyberhack · https://tokensand.com/cyberhack/projects · https://tokensand.com/vastsf/projects
- Winner pages: https://tokensand.com/p/nurserai · /p/vigil · /p/dead-reckoning · /p/livik · /p/local-ad-platform-lh · /p/sitelens · /p/local-news-anchor · /p/long-horizon-wave · /p/riff · /p/verity · /p/breakout · /p/swarmads · /p/seekr-see-find-remember
- Devpost organizer listing: https://devpost.com/hackathons?organization=Creators+Corner
- Devpost events: https://loop-engineering-hackathon.devpost.com/ · https://harness-hack.devpost.com/ · https://agentic-engineering-hack.devpost.com/ · https://ship-to-prod.devpost.com/ · https://multimodal-frontier-hackathon.devpost.com/ · https://deep-agents-hackathon.devpost.com/ · https://autonomous-agents-hackathon.devpost.com/ · https://self-improving-agents-hack.devpost.com/ · https://continual-learning-hackathon.devpost.com/ · https://agentic-orchestration-hack.devpost.com/ · https://production-agents-hackathon.devpost.com/ · https://self-evolving-agents-hack.devpost.com/ · https://the-future-of-agents-hackathon.devpost.com/ · https://nyc-ai-agents-hackathon.devpost.com/ · https://mcp-ai-agents-hackathon.devpost.com/ · https://a2a-agents-hackathon.devpost.com/ · https://mcp-aws-enterprise-agents.devpost.com/ · https://aws-mcp-agents-hackathon.devpost.com/ · https://mcp-and-a2a-hackathon.devpost.com/ · https://ai-agents-hackathon-gtc.devpost.com/ · https://multimodal-ai-agents.devpost.com/ · https://vertical-specific-ai-agents.devpost.com/ · https://voice-video-ai-agents.devpost.com/ · https://multiagents-hackathon.devpost.com/
- Devpost project pages linked inline (devpost.com/software/…)
- https://crunchbase.com/person/saroop-bharwani · https://ycombinator.com/companies/senso · https://x.com/saroopbharwani/status/1929572366277017666 · https://github.com/alessandroamenta
