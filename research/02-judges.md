# 02 — Judges

> Researched 2026-11-19 from public professional sources only. Lines marked **Inference** are my reading of what would land with that person, not something they have said. Where I could not confirm identity or details I say so. LinkedIn profiles were not readable, so career detail comes from personal sites, conference profiles, company pages and press.

## How the panel is built

20 judges are listed on [Luma](https://luma.com/personalagenthack). 12 work for sponsors and will mostly pick their own track winners. 8 do not work for a sponsor:

| Non-sponsor judge | Why they matter |
|---|---|
| Mogana Kumaran Sivaraman (Gap) | Only retail person on the panel, and a repeat judge: fifth tokens& event since June |
| Lingxi Li (Cursor) | Repeat judge (Loop Engineering, Jul 2026); design engineer |
| Riya Dashoriya (Meta) | Judged a 2025 event by the same organizer; frequent hackathon judge |
| Robert Farlow (Plato) | Builds simulated environments to test agents — professionally sceptical of agent demos |
| AJ Jain and Aditya Ketkar (Broccoli) | Run agents that take real bookings for 1,500+ businesses |
| Michael Moyo (Ocular AI) | Founder; data quality for AI |
| Harshal Vaidya (Intuit) | Staff engineer at a consumer-finance company |

Past events had a separate "General Judge" or "Overall" prize alongside sponsor tracks ([Loop Devpost](https://loop-engineering-hackathon.devpost.com/)). **Inference:** these eight carry the overall decision, with sponsor judges scoring finalists too.

Three shared traits among the eight: several run or test agents that act in the real world (Plato, Broccoli, Firedrill on the sponsor side), so they know how agent demos fail; two are data-reliability people (Mogana, Moyo); two are front-end and design people (Lingxi, Riya).

---

## Non-sponsor judges (deep profiles)

### Mogana Kumaran Sivaraman — Sr Staff Software Engineer, Gap Inc.

- **Role:** Listed as Sr Staff Software Engineer at Gap Inc. for this event; listed as "Senior Staff Data Engineer @ Gap Inc" at four earlier tokens& events ([Harness, Jun 12](https://luma.com/harnesshack), [Loop, Jul 17](https://luma.com/loophack), [Self-Evolving, Jul 24](https://luma.com/swarmhack), [Long Horizon, Sep 25](https://luma.com/horizonagentshack)). He is the most consistent non-sponsor judge in this series.
- **Background:** Data platform engineering at Gap, San Francisco. His GitHub portfolio lists "technical publications, peer review, and applied AI judging" as focus areas ([GitHub](https://github.com/moganakumaran/portfolio)).
- **Public writing:**
  - "The Reliability Gap in Modern Lakehouse Architectures: A Failure Fingerprinting Framework for Operational Intelligence" — argues platforms can reconstruct a failure in detail but cannot recognise it as a repeat of a solved one; proposes hashing each failure episode to a stable SHA-256 identifier in a queryable registry ([journal](https://journalcjast.com/index.php/CJAST/article/view/4673)).
  - "Historical Incident Reuse for Data Platform Reliability" ([PDF](https://www.jisem-journal.com/download/13_Mohan-03-Historical_Incident_Reuse.pdf)).
  - "DARE: Separating Epistemic Disagreement from Action Risk in Multi-Agent Failure Recovery" ([ResearchGate](https://www.researchgate.net/publication/415823406_DARE_Separating_Epistemic_Disagreement_from_Action_Risk_in_Multi-Agent_Failure_Recovery)).
  - "Semantic Data Quality for Agentic DataOps", registered at ICIEEE 2026 with Gap Inc. affiliation ([registration list](https://registration.icieee.com/)).
  - A manuscript on operational metadata as a property graph ([GitHub](https://github.com/moganakumaran/operational-metadata-property-graph)).
- **Technical interests:** Data quality, lineage, operational metadata, reusing past incidents, and — from the DARE title — separating "the agents disagree about what is true" from "this action is risky".
- **Caveat:** His public work is data-platform engineering. I found nothing tying him to Gap's storefront, checkout or merchandising teams, so do not assume storefront expertise. He will still know retail data: SKUs, inventory feeds, price and promotion data, returns.
- **Inference — what would impress him:**
  - An audit trail that is a **queryable data model**, not a pretty transcript: stable IDs per decision, the exact source version each fact came from, replayable.
  - **Data freshness handled explicitly**: price and stock re-checked before the action, with a visible "stale, re-verifying" state. Retail data goes out of date by the hour.
  - **Risk-gated action**: the agent distinguishes "I am unsure which product is right" from "this purchase is hard to undo" and treats them differently. That is the DARE idea applied to shopping.
  - Realistic retail mess: size and variant mismatches, out-of-stock substitutions, a return window that differs by item category, a promo that excludes the item.
  - He scored the Long Horizon event where the overall winner was permission-bounded (NurserAI) and a track winner survived a mid-payment crash without double-paying (Dead Reckoning) ([gallery](https://tokensand.com/horizonagentshack/projects)). Idempotent purchase handling is a proven hit with a panel he sat on.

### Lingxi Li — Software Engineer, Cursor

- **Role:** Listed as Software Engineer at Cursor. Personal site: "Builds the future of coding at Cursor", working on **Glass**, described as product engineering, design engineering and systems work to make agentic coding "faster, more reliable, and more contextual" ([lingxi.li](https://lingxi.li/)). GitHub bio: Software Engineer at Cursor, product and design engineering ([GitHub](https://github.com/lilingxi01)).
- **Possible change, unverified:** Third-party pages describe him as now a core engineer and mobile lead for "Grok Bot" at SpaceXAI, and a write-up says he runs 200+ cloud agents through five specialised bot instances ([alphaXiv profile](https://www.alphaxiv.org/@lingxi-li-2), [explainx write-up](https://explainx.ai/blog/grok-bot-engineering-org-200-cloud-agents-lingxi-li-2026)). tokens& galleries from October tag the sponsor as "SpaceXAI (Cursor)" ([gallery](https://tokensand.com/vastsf/projects)), so the two may be the same organisation now. Luma still says Cursor. Do not raise this with him; just know he may operate large agent fleets day to day.
- **Background:** BS and MS in computer science, UMass Amherst; earlier led front-end at Contextual AI (per alphaXiv, self-published and unverified). Interested in reinforcement learning for UI generation ([lingxi.li](https://lingxi.li/)).
- **Self-description:** "A solution shipper, who cares deeply about both engineering and design"; "Renaissance Engineer"; "build once and run forever" — design for scale from the start.
- **Prior judging:** Loop Engineering Hackathon, Jul 17, 2026 ([Luma](https://luma.com/loophack)). The General Judge winner there was a tightly scoped infrastructure tool using 3 sponsor products ([Devpost](https://devpost.com/software/fleet-yield-middleware)).
- **Inference — what would impress him:**
  - **Craft in the interface.** He is the judge most likely to notice typography, motion, loading states, and whether the audit trail is a designed surface or a JSON dump. "Beyond a chat window" is his home ground.
  - An agent UI that shows state the way a good coding agent does: plan, steps, diffs, approve or reject. A **"diff" of the cart before and after the agent acted** would read as native to him.
  - Speed and reliability on stage. No spinner for 40 seconds.
  - Clean architecture he can see in the repo in two minutes.

### Riya Dashoriya — Engineering Manager, Meta

- **Role:** Engineering Manager at Meta per her LinkedIn headline and Google Scholar listing ([LinkedIn](https://www.linkedin.com/in/riyadashoriya/), [Scholar](https://scholar.google.com/citations?user=QjwYxnwAAAAJ&hl=en)). I could not find which Meta team — **not found**.
- **Background:** Previously Engineering Manager and software engineer at Quizlet; MS in Information Systems, University of Utah; Women Techmakers Ambassador; organiser of GDG San Francisco and GDG Cloud San Francisco ([Sessionize](https://sessionize.com/riya/)). Writes on Medium as Reeyah S Dasoriya ([Medium](https://medium.com/@reeyahsdasoriya)); podcast episode on speaking at global conferences ([NinjaTalks](https://creators.spotify.com/pod/profile/theninjatalks/episodes/On-a-journey-to-speaking-at-global-conferences--Riya-Dashoriya--NinjaTalks-Episode-07-efoj04)).
- **Talks:** "Getting Started with Accessibility" (Quizlet's path to WCAG Level A), "11 ways to make your website #a11y compliant", "Chrome DevTools 101", "Actions on Google" workshops, several talks on inclusive events and career growth ([Sessionize](https://sessionize.com/riya/)).
- **Prior judging:** A2A Agents Hackathon by the same organizer ([Luma](https://luma.com/aoc5opn4)), Afore x AI Valley Agent Foundry ([Devpost](https://afore-agent-foundry.devpost.com/)), Worlds in Action Hack ([Devpost](https://world-model-hackathon.devpost.com/)). She has seen a lot of agent demos.
- **Technical interests:** Web front-end (React, JavaScript, CSS), accessibility, web performance, developer tools, voice interfaces, community and mentoring.
- **Inference — what would impress her:**
  - **A named human user with a real constraint**, especially one that mainstream shopping serves badly: low vision, limited mobility, a carer buying for someone else, a fixed budget. Accessibility is the most documented theme of her public work.
  - An interface that works with keyboard and screen reader, readable contrast, plain language in the audit trail. Saying "the trail is readable by a screen reader" in the demo would be noticed.
  - A clear, well-paced presentation. She coaches people on public speaking.
  - As a manager: evidence the team scoped sensibly and can explain trade-offs.

### Robert Farlow ("Robert F.") — Co-founder & CEO, Plato

- **Identification:** "Robert F., Co-founder & CEO @ Plato" matches **Robert Farlow**, co-founder and CEO of Plato, the San Francisco company building simulated workplaces for AI agents ([The Next Web](https://thenextweb.com/news/plato-16m-series-a-simulated-workplaces-ai-agents), [Yahoo Finance press release](https://finance.yahoo.com/technology/ai/articles/plato-raises-16-million-build-131700376.html)). Confidence: high. There are other companies called Plato (one has a co-founder and CEO named Benedikt Nolte, per [Crunchbase](https://www.crunchbase.com/person/benedikt-nolte-1436)); the first name and initial rule those out.
- **What Plato does:** Rebuilds the software and tasks of real jobs — customer support, finance, accounting, HR, healthcare admin — as simulations, paying professionals to recreate their own roles. AI labs train and test agents there without touching live systems or real money. Environments can be repeated, modified and reset. Plato **checks the process as well as the final result, including rule-following, so agents cannot score well by gaming the test** ([The Next Web](https://thenextweb.com/news/plato-16m-series-a-simulated-workplaces-ai-agents)). An industry directory describes it as replica-website reinforcement-learning environments and evals for browser and computer-use agents ([rl-list](https://www.rl-list.com/vendors/plato)).
- **Company facts:** Founded 2025; $16M Series A announced Oct 28, 2026, led by Nava Ventures with Gradient Ventures, Transpose Platform and Mustafa Suleyman; $19M total. Co-founder and CTO Pranav Putta (previously MultiOn, per rl-list).
- **Quote:** "Mastering real complex work comes from experience doing those tasks."
- **Personal background:** Earlier career **not found**.
- **Why he might be judging:** Plato's customers are teams building agents; hackathons are where those teams and future hires are. The theme (agents taking actions, with a trail of how) is Plato's evaluation thesis.
- **Inference — what would impress him:**
  - **Process correctness, not only outcome.** He will ask "did it follow the rules on the way there?" Show the policy the agent was bound by and where each step was checked against it.
  - A **resettable sandbox retailer** with seeded edge cases, and the agent run against several scenarios — including ones where the right answer is "do not buy". This is his company's product in miniature.
  - A pass/fail scorecard across scenarios ("7 of 8 scenarios correct; here is the one it got wrong and why").
  - Honesty about what is simulated. He builds simulations; he will respect a clearly labelled one and distrust a fake that pretends to be live.
  - Evidence the agent cannot game the check — for example the verifier is separate from the agent.

### AJ Jain (CEO) and Aditya "Adi" Ketkar (CTO) — Broccoli

- **Identification:** **Broccoli AI** (YC W22), San Francisco: AI agents for home-service businesses. Confidence: high — press names AJ Jain as CEO and Adi/Aditya Ketkar as CTO ([Fortune](https://fortune.com/2026/10/28/broccoli-ai-funding-khosla-ventures-marcelle-flowers-plumbing-gen-z-founders/), [Contracting Business](https://www.contractingbusiness.com/industry-news/news/55415468/broccoli-raises-52-million-to-expand-ai-for-home-services)). Some databases list the CEO as "Abhishek Jain" ([Crunchbase](https://www.crunchbase.com/organization/broccoli-db01)) — likely his full name, unverified.
- **What Broccoli does:** Voice and text agents for plumbing, HVAC, electrical, roofing and similar contractors. They answer inbound calls around the clock, **book jobs directly into the scheduling software the contractor already uses** (ServiceTitan integration), follow up on open estimates and renew service memberships ([Fortune](https://fortune.com/2026/10/28/broccoli-ai-funding-khosla-ventures-marcelle-flowers-plumbing-gen-z-founders/), [pulse2](https://pulse2.com/broccoli-raises-52-million-across-series-a-and-series-b-led-by-khosla-ventures/)).
- **Numbers:** $52M across Series A and B led by Khosla Ventures (announced late Oct 2026); 1,500+ businesses including Ace Hardware Home Services and Mr. Electric; company says 10M+ conversations and 2.5M appointments booked; headcount 15 → 150+ in a year.
- **Founders:** Met at BITS Pilani in 2016; building companies together since 2017 (campus delivery app, interview-prep bootcamp); went through YC with Telematica, a universal API for electric vehicles, which did not scale; pivoted. Built the first version for a Sacramento plumbing company in 10 days after learning its answering service cost $5,000 a month and booked 33% of calls. That owner, Marcelle Flowers, is now an executive at Broccoli ([Fortune](https://fortune.com/2026/10/28/broccoli-ai-funding-khosla-ventures-marcelle-flowers-plumbing-gen-z-founders/)).
- **Stated philosophy:** AI as teammate; roughly 80% handled by the agent, **human judgment for the rest, emergencies handed to staff**; "business in a box". Jain appears on trades podcasts ([Born For The Trades](https://www.buzzsprout.com/2149153/episodes/17891310-from-the-trenches-to-tech-aj-jain-s-mission-to-revolutionize-the-trades-with-ai)).
- **Why they might be judging:** Fresh funding and fast hiring (the YC jobs page lists a staff engineer / tech lead role). They ship agents that complete a transaction-like action for non-technical customers, which is the theme.
- **Inference — what would impress AJ Jain (CEO):**
  - A **non-tech, non-SF customer** with a problem that costs money. This matches both his company and the organizers' "think beyond the problems in SF". A homeowner replacing a failed water heater today is closer to his world than a founder buying a monitor.
  - A business number: dollars saved, conversion, time to completion.
  - **The action lands in a real system of record** (an order, a booking, a calendar entry), not a "success" toast.
  - Clear hand-off to a human when the agent should not proceed.
  - Voice or phone as the interface would feel familiar — but only if it works flawlessly.
- **Inference — what would impress Aditya Ketkar (CTO):**
  - Integration depth and failure handling: retries, idempotency, what happens when the retailer API returns a different price than the one quoted.
  - Latency. Voice-agent builders care about response time.
  - Per-step traces that an on-call engineer could debug from. His team must already answer "why did the agent book that?" for customers.

### Michael Moyo — CEO & Co-Founder, Ocular AI (YC W24)

- **Identification:** Confirmed — CEO and co-founder of Ocular AI, YC Winter 2024 ([YC](https://www.ycombinator.com/companies/ocular-ai)).
- **What Ocular AI does:** Launched in YC W24 as open-source enterprise search plus a copilot across workplace tools with role-based permissions; has since repositioned. Current YC tagline: "Applied AI Data Research Lab Encoding Human Expertise into Machines" — data labelling and expert data for training AI ([YC](https://www.ycombinator.com/companies/ocular-ai), [Seedtable](https://seedtable.com/companies/ocular-ai)). Team of about 8; hiring founding engineers and a founding SDR.
- **Background:** Computer and biomedical engineer; Dartmouth; former software engineer at Microsoft; from Zambia; earlier co-founded a fintech (Ipahive) and a mental-health venture in Zambia, per profiles ([itedgenews](https://www.itedgenews.africa/two-african-co-founders-behind-ocular-ai-game-changing-startup-at-y-combinators-forefront/), [zerogtalent](https://zerogtalent.com/people/michael-moyo)). Co-founder and CTO Louis Murerwa is ex-Google. A 2024 article reported a move to Atlanta ([Hypepotamus](https://www.hypepotamus.com/enterprise-focused-startup-building-the-search-for-everything-platform-relocates-to-atlanta/)); YC lists San Francisco.
- **Why he might be judging:** Founder visibility and hiring; his company's thesis is that AI is only as good as the human-verified data behind it, which is the "reliable context" clause of the challenge.
- **Inference — what would impress him:**
  - **Provenance of context**: where each fact came from, who vouches for it, how confident the system is, and a human-correction loop that improves the next run.
  - Permission-aware retrieval (his first product enforced role-based access): the agent only uses data it is allowed to.
  - "Think beyond SF" will resonate personally — a shopper in a market with unreliable delivery, cross-border fees or mobile-money payment would stand out. Only do this if you can make it credible.
  - Scrappy but real: an actual working flow over slides.

### Harshal Vaidya — Staff Software Engineer, Intuit

- **Identification:** A LinkedIn profile for Harshal Vaidya at Intuit exists ([LinkedIn](https://www.linkedin.com/in/harshalsunilvaidya/)); aggregator profiles describe a Staff Software Engineer at Intuit in San Diego with an MS in computer science from San Diego State University and an earlier stint as Software Engineer 2 at Intuit around 2016–2018 ([RocketReach](https://rocketreach.co/harshal-vaidya-email_61554275)). Confidence: medium — several unrelated people share the name (Intuitive Surgical, Cognizant and others), and the detail above is from aggregators. Team at Intuit, public talks and writing: **not found**.
- **Context:** Intuit makes TurboTax, QuickBooks, Credit Karma and Mailchimp — products where software handles people's money and a wrong automated action has real consequences. Intuit's public positioning is "done-for-you" AI agents with expert review. I could not confirm which of these he works on.
- **Inference — what would impress him:**
  - Treating a purchase as a **financial transaction**: spending limits, explicit authorisation, confirmation before commit, a receipt, and reversibility (cancel window, return path).
  - Staff-engineer concerns: clear system boundaries, idempotent payment calls, secrets handled properly, tests or evals in the repo.
  - A ledger-like audit trail (who authorised what, for how much, when) will read as familiar to someone from a tax and accounting company.
  - Explainability a non-expert can follow — Intuit's products explain "why" to consumers.

---

## Sponsor-employed judges (short profiles)

Other researchers cover the products. These notes are about the person.

| Judge | Role | Notes | Inference: what lands |
|---|---|---|---|
| **Daniel Svonava** | Co-founder & CEO, Superlinked | Founded Superlinked in 2021; previously at Google working on ML infrastructure for YouTube Ads forecasting and pricing; regular conference speaker on vector search and retrieval ([Superlinked](https://www.superlinked.com/about), [AI Infra Summit](https://www.ai-infra-summit.com/speakers/daniel-svonava), [DataTalks](https://datatalks.club/people/danielsvonava.html)). Also a speaker tomorrow. | Retrieval that mixes structured attributes (price, size, rating, recency) with text, not plain text embeddings. Product search is his canonical use case — show ranking that respects hard constraints. |
| **Rahul Parundekar** | DevRel, Superlinked | Listed on Luma. An MLOps Community speaker page names a Rahul Parundekar as founder of AI Hero, MS CS USC 2010 ([MLOps Community](https://home.mlops.community/public/videos/declarative-mlops-streamlining-model-serving-on-kubernetes)); likely the same person, **unverified**. | Clean, well-documented use of SIE that he could show others; a repo readable enough to become an example. |
| **Anneketh Vij** | AI Research Scientist, Arcee AI | Co-author on Arcee papers: domain adaptation of Llama-3 70B via continual pre-training and model merging; the Trinity Large technical report; wrote the Trinity Builders Program post ([Hugging Face](https://huggingface.co/annekethvij/papers), [CatalyzeX](https://www.catalyzex.com/author/Anneketh%20Vij)). | A reasoned model choice with numbers: which model for which step, cost and latency against quality. The track is literally "Pareto frontier". |
| **Sam Fraser** | Product, Arcee AI | Director of Product at Arcee; previously product manager at Microsoft (2021–2024) and data scientist ([The Org](https://theorg.com/org/arcee-ai/org-chart/sam-fraser)). Speaker tomorrow. | A product story for why small or open-weight models fit shopping (cost per task, privacy, running many checks cheaply). |
| **Veljko Kovac** | Head of FDE, Cognee | Writes Cognee's blog on memory for support agents and structured memory for agent workflows; speaking at DevCon Fall 2026 on deterministic memory for coding agents ([Cognee author page](https://www.cognee.ai/authors/veljko-kovac), [Tessl DevCon](https://tessl.io/devcon/speakers)). Speaker tomorrow. | Memory that is structured and disambiguated (this shopper's size in this brand; "Mom's address" versus "home"), and that visibly changes a later decision. |
| **Corbett Waddingham** | Head of Developer Relations, Guild.ai | Appears as "Cory Waddingham, Head of Developer Relations, Guild.ai" on Sessionize ([Sessionize](https://sessionize.com/waddingham/)) — same title, so very likely the same person. Judged the Cyberdefense hack on Oct 9 ([Luma](https://luma.com/cyberhack)). Guild's CEO is James Everingham, formerly Meta developer infrastructure; Guild positions itself as a control plane for agents ([GV](https://gv.com/news/guild-ai-sharing-intelligence)). Speaker tomorrow. | A past Guild-track winner centred on governed autonomy: DailyGate "earns its autonomy" from approvals ([DailyGate](https://devpost.com/software/dailygate)). The Oct 9 Guild prize was worded "Best use of Guild to host and run agents" ([page](https://tokensand.com/cyberhack)). Agents actually hosted and run on Guild, with permissions that change based on behaviour. |
| **Richelle Herrli** | Senior Developer Advocate, Airbyte | Writes Airbyte's blog: release overviews, a LangChain agent guide, sync logs ([Airbyte author page](https://airbyte.com/blog-authors/richelle-herrli)). Speaker tomorrow. | Real connectors pulling the shopper's own data (order history, email receipts, calendar) as context — "Conquer with Context" was the track name at earlier events. |
| **Newton Asare** | Co-founder & CEO, Firedrill | Second company with co-founder Kiran Das after an acquired startup (ClientDesk); earlier roles at Reload, Epic and Scotiabank per his profile ([X](https://x.com/newtonasareII), [TechCrunch on Reload](https://techcrunch.com/2026/02/19/reload-an-ai-employee-agent-management-platform-raises-2-275m-and-launches-an-ai-employee/)). Firedrill supplies synthetic tools and test data so teams can run agent scenarios and inspect what changed without touching live systems. | Same instinct as Plato's Farlow: scenario runs against a synthetic retailer, before/after state diffs, failures found and fixed during the hack. |
| **Rebecca Abu Sharkh** | Solutions Engineering, Amplitude | Mentioned by name in an Amplitude staff profile as a solutions engineer who brought a colleague into the field ([Amplitude blog](https://amplitude.com/blog/meet-the-ampliteer-bethany-mangan)). Little else public. | Meaningful event instrumentation of the agent funnel (intent → options shown → approval → purchase → return), plus a flag or experiment that changes agent behaviour. |
| **Kerwell L.** | Product Marketing Manager, Persona | Surname not published; **not identified** beyond the Luma listing. Speaker tomorrow. | A crisp story for why identity matters when an agent buys: verifying the human behind the agent before a high-value or age-restricted purchase. |
| **Amanda Kwong** | Solutions Engineer, SingleStore | No public profile found beyond the listing. Speaker tomorrow. | SingleStore doing something a plain Postgres would not: transactions, vectors and analytics on the audit log in one place, queried live in the demo. |
| **Ryan Sarginson** | Solutions Engineer, SingleStore | No public profile found beyond the listing. | Same as above; a real-time query over the decision log during the demo. |

---

## What this means for the build

**Inference, drawn from the profiles above:**

1. **Process over outcome.** Farlow (Plato), Asare (Firedrill) and Mogana (Gap) all work on whether a system did the right thing the right way. Show rule checks at each step, not just a successful order.
2. **Show a failure on purpose.** Stale price, out-of-stock item, a policy that forbids the purchase, a crash after payment. A past panel including Mogana rewarded exactly this.
3. **Pick a shopper who is not a San Francisco engineer.** Jain, Ketkar, Moyo and Dashoriya each have a public record pointing to non-tech or underserved users, and the organizers asked for it.
4. **Design the interface.** Lingxi Li and Riya Dashoriya are front-end people. A designed, accessible audit-trail view beats raw logs.
5. **Treat money seriously.** Vaidya (Intuit) and Ketkar will look for limits, authorisation, idempotency and a clear real / test / simulated label.
6. **Give numbers.** Scenarios passed, dollars saved, seconds to complete, cost per run.
7. **The action should land somewhere real** — an order in a store admin, a payment in a Stripe dashboard, a calendar hold — and be shown there.

## Could not verify

- Riya Dashoriya's team at Meta; any 2025–2026 talks.
- Whether Lingxi Li is currently at Cursor or at SpaceXAI (sources conflict; Luma says Cursor).
- Robert Farlow's career before Plato.
- Harshal Vaidya's team at Intuit and any public writing; identity is medium confidence.
- Whether Mogana Kumaran Sivaraman has worked on Gap's storefront or checkout systems.
- Kerwell L.'s surname; public profiles for Amanda Kwong, Ryan Sarginson and Rebecca Abu Sharkh.
- Whether Rahul Parundekar of Superlinked is the AI Hero founder of the same name.
- How the overall prize is actually decided (which judges vote on it).

## Sources

- https://luma.com/personalagenthack · https://luma.com/loophack · https://luma.com/harnesshack · https://luma.com/swarmhack · https://luma.com/horizonagentshack · https://luma.com/cyberhack · https://luma.com/aoc5opn4
- Mogana: https://github.com/moganakumaran/portfolio · https://github.com/moganakumaran/operational-metadata-property-graph · https://journalcjast.com/index.php/CJAST/article/view/4673 · https://www.jisem-journal.com/download/13_Mohan-03-Historical_Incident_Reuse.pdf · https://www.researchgate.net/publication/415823406_DARE_Separating_Epistemic_Disagreement_from_Action_Risk_in_Multi-Agent_Failure_Recovery · https://registration.icieee.com/ · https://tokensand.com/horizonagentshack/projects
- Lingxi Li: https://lingxi.li/ · https://github.com/lilingxi01 · https://www.alphaxiv.org/@lingxi-li-2 · https://explainx.ai/blog/grok-bot-engineering-org-200-cloud-agents-lingxi-li-2026 · https://tokensand.com/vastsf/projects · https://devpost.com/software/fleet-yield-middleware
- Riya Dashoriya: https://sessionize.com/riya/ · https://www.linkedin.com/in/riyadashoriya/ · https://scholar.google.com/citations?user=QjwYxnwAAAAJ&hl=en · https://medium.com/@reeyahsdasoriya · https://creators.spotify.com/pod/profile/theninjatalks/episodes/On-a-journey-to-speaking-at-global-conferences--Riya-Dashoriya--NinjaTalks-Episode-07-efoj04 · https://afore-agent-foundry.devpost.com/ · https://world-model-hackathon.devpost.com/
- Plato: https://thenextweb.com/news/plato-16m-series-a-simulated-workplaces-ai-agents · https://finance.yahoo.com/technology/ai/articles/plato-raises-16-million-build-131700376.html · https://www.rl-list.com/vendors/plato · https://www.crunchbase.com/person/benedikt-nolte-1436
- Broccoli: https://fortune.com/2026/10/28/broccoli-ai-funding-khosla-ventures-marcelle-flowers-plumbing-gen-z-founders/ · https://www.contractingbusiness.com/industry-news/news/55415468/broccoli-raises-52-million-to-expand-ai-for-home-services · https://pulse2.com/broccoli-raises-52-million-across-series-a-and-series-b-led-by-khosla-ventures/ · https://www.crunchbase.com/organization/broccoli-db01 · https://www.buzzsprout.com/2149153/episodes/17891310-from-the-trenches-to-tech-aj-jain-s-mission-to-revolutionize-the-trades-with-ai
- Ocular AI: https://www.ycombinator.com/companies/ocular-ai · https://seedtable.com/companies/ocular-ai · https://www.itedgenews.africa/two-african-co-founders-behind-ocular-ai-game-changing-startup-at-y-combinators-forefront/ · https://zerogtalent.com/people/michael-moyo · https://www.hypepotamus.com/enterprise-focused-startup-building-the-search-for-everything-platform-relocates-to-atlanta/
- Harshal Vaidya: https://www.linkedin.com/in/harshalsunilvaidya/ · https://rocketreach.co/harshal-vaidya-email_61554275
- Sponsor judges: https://www.superlinked.com/about · https://www.ai-infra-summit.com/speakers/daniel-svonava · https://datatalks.club/people/danielsvonava.html · https://home.mlops.community/public/videos/declarative-mlops-streamlining-model-serving-on-kubernetes · https://huggingface.co/annekethvij/papers · https://www.catalyzex.com/author/Anneketh%20Vij · https://theorg.com/org/arcee-ai/org-chart/sam-fraser · https://www.cognee.ai/authors/veljko-kovac · https://tessl.io/devcon/speakers · https://sessionize.com/waddingham/ · https://gv.com/news/guild-ai-sharing-intelligence · https://devpost.com/software/dailygate · https://tokensand.com/swarmhack/projects · https://airbyte.com/blog-authors/richelle-herrli · https://x.com/newtonasareII · https://techcrunch.com/2026/02/19/reload-an-ai-employee-agent-management-platform-raises-2-275m-and-launches-an-ai-employee/ · https://amplitude.com/blog/meet-the-ampliteer-bethany-mangan
