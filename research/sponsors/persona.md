# Persona — sponsor research (Personal Agents Hackathon, Nov 20 2026)

> Researched Nov 19 2026. Every non-obvious claim has an inline source. "Unverified" / "not found" means exactly that.
> Note on sources: `withpersona.com` marketing pages return HTTP 403 to automated fetches; the same pages were read from the mirror host `marketing.withpersona.com`. Docs (`docs.withpersona.com`) and Help Center (`help.withpersona.com`) serve clean Markdown if you append `.md` to any URL, and both publish an `llms.txt` index.

---

## 1. TL;DR

- **What it is:** Persona (Persona Identities, Inc., San Francisco) is an identity-verification platform: government ID + selfie liveness + database checks + passive device/network/behaviour signals, orchestrated through "Inquiries", Workflows and Cases. $200M Series D at a $2B valuation in April 2025 ([Series D post](https://marketing.withpersona.com/blog/series-d)).
- **Its agent-era pitch is "Know Your Agent" (KYA):** not a separate API, but a framework + a set of existing primitives (Inquiries, Relay, Signals/Sentinel, Accounts, Workflows) for answering three questions: is this an agent or a human, who is the human behind it, can that human be trusted ([KYA solution page](https://marketing.withpersona.com/solutions/know-your-agent/); [Help Center: Verifying AI Agents](https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/)).
- **What the prize wants:** "Best Use of Persona" under the tagline "Detect agents and verify the people behind them" ([event page](https://tokensand.com/personalagenthack)). No criteria are published. Persona's own writing says agentic commerce needs **origin, consent with scope, differential risk (step-up), and auditability** ([Jan 2026 post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv)) — which maps almost one-to-one onto "shopping agent with an audit trail".
- **Fastest qualifying integration (about 30–45 min):** self-serve sandbox signup, add the "KYC" solution, `POST /api/v1/inquiries` from your backend with a reference ID, open `https://inquiry.withpersona.com/verify?inquiry-id=inq_...` (or the embedded JS SDK), then poll `GET /api/v1/inquiries/{id}` for `approved` and gate the agent's purchase authority on it. Sandbox does not run real verification; a "Pass verifications" toggle forces pass/fail ([Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api)).
- **Best way to win:** build exactly the lifecycle Persona documents — **verify the person, then link the agent, then step-up re-verify the human on a risky purchase, and log the Persona inquiry ID/status as an audit-trail entry** — and keep PII out of the LLM (pass the model only `verified` / `not_verified`), which is Persona's explicit guidance ([Verify users directly in chat](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).

---

## 2. What they do

### Product surface

| Layer | What it is | Source |
|---|---|---|
| **Inquiries** | One instance of a person going through a verification flow, created from an Inquiry Template. Statuses: `created`, `pending`, `completed`, `failed`, `expired`, `needs_review`, `approved`, `declined`. | [Create an Inquiry API ref](https://docs.withpersona.com/2026-09-29/api-reference/inquiries/create-an-inquiry), [Accessing Inquiry status](https://docs.withpersona.com/2026-09-29/accessing-inquiry-status) |
| **Verifications** | Government ID (200+ countries/territories), Selfie/liveness, Database, Document, Phone, Email, NFC, AAMVA, etc. | [docs index](https://docs.withpersona.com/2026-09-29/llms.txt), [KYA page](https://marketing.withpersona.com/solutions/know-your-agent/) |
| **Integration methods** | Hosted Flow (a link), Embedded Flow (JS SDK iframe/modal), iOS/Android/React Native SDKs. | [Getting started](https://docs.withpersona.com/2026-09-29/getting-started) |
| **Relay** | Privacy-preserving verification built on IETF Privacy Pass. Returns only a claim result (e.g. `live_human_presence`, Age 18+/21+ Verified), no identity data; Persona deletes the underlying data and cannot see which platform asked ("double-blind"). | [Relay overview](https://docs.withpersona.com/2026-09-29/relay), [Relay schemas](https://docs.withpersona.com/2026-09-29/relay-schemas) |
| **Signals** | 50+ passive signals collected during an Inquiry: network (proxy, Tor, IP threat), behavioural (**Bot Score**, shortcut usage, hesitation), device (integrity, fingerprint), image (virtual camera risk). | [Signals overview](https://help.withpersona.com/articles/3aYYkI9NMDsyVqx5V3IT3G/) |
| **Sentinel** | Passive device/network signal collection *outside* a verification flow (sign-in, payout change, etc.), recorded as Transactions that trigger Workflows which can allow, hold, open a Case, or trigger step-up verification. | [Sentinel overview](https://help.withpersona.com/articles/OzIPtG0LQsMTVqp22x0t4Q/) |
| **Accounts / Reference IDs** | One record per end user tying together all their inquiries and verifications; keyed by your own user ID. | [Reference IDs](https://docs.withpersona.com/2026-09-29/reference-ids) |
| **Workflows, Cases, Graph, Reports** | Decision automation, manual review, link analysis, watchlist/PEP/adverse-media reports. | [docs index](https://docs.withpersona.com/2026-09-29/llms.txt) |
| **Verify with Persona** | Reuses a returning user's past Persona verifications across the network as evidence, with consent. **Closed beta** — not hackathon-usable. | [Help Center](https://help.withpersona.com/articles/4Yl6a7VEH2ScPfAHDif0Ts/) |
| **Persona AI / Persona Extend** | Persona's own AI (Policy Agents, Case Review Agents) plus an **MCP server** that lets Claude/ChatGPT/your agent read Persona records and create inquiries. | [Connect AI agents to Persona with MCP](https://help.withpersona.com/articles/3Ev9FHB3kXENDUwbGg20KX/) |

### The agent-era offering, precisely

There is **no dedicated "KYA API", agent credential, agent token, or delegated-authority protocol** in the public docs. The docs index has no agent-specific endpoints ([docs index](https://docs.withpersona.com/2026-09-29/llms.txt)), and the KYA solution page names no protocol such as AP2, ACP, Visa TAP or Web Bot Auth ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)). Persona's own explainer says there are "some outstanding questions about what will become best practices and shared protocols for KYA" ([What is KYA](https://marketing.withpersona.com/blog/what-is-know-your-agent-kya)).

What exists is:

1. **"Know Your Agent" solution page** — headline "Verified identity for the modern agentic landscape"; three questions (agent or human? who is behind it? can they be trusted?); two lifecycle moments, **Onboarding** (verified human identity is the trust signal that sets the agent's scope) and **Reverification** (unblock high-risk actions in real time by re-verifying the human bound to the agent); three primitives: Government ID + selfie liveness, **Relay**, and **Passive signals** ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)).
2. **Help Center article "Verifying AI Agents"** (last updated Nov 7 2026) — the most concrete guidance. Three inputs per agent request (the action, the agent, the person); four "layers of identity" (Humanness, Eligibility, Uniqueness, Identity proofing); a four-step ladder: agent signals only → a single claim → full identity check → full identity check **plus the person's explicit approval of the action, which "your product collects and records"**. And a procedure: verify the person, create an Account, build a Workflow, send them through the Inquiry, then "link the agent only after the result ... in your own product and limit what it can do to what you verified." It also states plainly: "Persona does not control the agent." ([Verifying AI Agents](https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/))
3. **Docs guide "Verify users directly in chat"** — how to run an Inquiry from inside a chat assistant using Device Handoff (QR code to phone), with the rule that the model never starts the verification and only ever receives `{ "identity_verification": "verified" }` ([docs](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).
4. **Relay "Live Human Presence" claim** — proof of humanness without PII ([Help Center](https://help.withpersona.com/articles/ytHNjFfek8K22wv85UPgVS/)).
5. **Persona MCP server** — V2 at `https://withpersona.com/mcp` (OAuth with Dashboard sign-in; reads records, can create an Inquiry with a one-time link and approve/decline inquiries; 20 changes/min; every change recorded in the org audit log) and V1 at `https://mcp.withpersona.com` (API key; every Persona API action as a tool) ([Help Center](https://help.withpersona.com/articles/3Ev9FHB3kXENDUwbGg20KX/)).

"Detect agents" in the hackathon tagline corresponds to passive signals (Bot Score, behaviour threat level, device integrity) and Sentinel — see section 7 for the catch (signals on the API are feature-flagged).

### Positioning

- Series D framing: "The real hurdle is no longer spotting bots; it's verifying who's behind every action and whether they can be trusted"; ambition to be "the verified identity layer for the internet" ([Series D post](https://marketing.withpersona.com/blog/series-d)).
- Footer tagline: "Humanizing online identity" ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)).
- Gartner Peer Insights 4.5 from 70 reviews as of July 7 2026 ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)); a secondary profile says Gartner Magic Quadrant Leader for Identity Verification two years running (unverified against Gartner; via search result summary of [komo.ai](https://komo.ai/directory/persona)).

### Competitors

- KYA-branded rivals found in this research: **Vouched** (Know Your Agent, May 2025), **Incode** (KYA binding agents to a face-verified person and spending mandate), **Prove** (KYA linking agent actions to verified users), **World** (AgentKit, April 17 2026 — proof an agent acts for a unique human) ([Vouched PR](https://www.silicon.co.uk/press-release/vouched-launches-know-your-agent-verification-to-bring-trust-and-identity-to-the-next-generation-of-ai-agents), [Incode coverage](https://idtechwire.com/incode-launches-know-your-agent-binding-ai-agents-to-a-face-verified-person-and-a-spending-mandate/), [Prove blog](https://www.prove.com/blog/agentic-commerce-identity-crisis-kya), [paz.ai on World AgentKit](https://www.paz.ai/blog/agentic-commerce-trust-stack-identity-layer)).
- Classic IDV competitors (general knowledge, not sourced in this pass): Jumio, Onfido/Entrust, Veriff, Socure, Sumsub, Stripe Identity, Plaid Identity Verification.

---

## 3. Company facts

- **Founded:** October 2018, San Francisco, by **Rick Song (CEO, ex-Square)** and **Charles Yeh (CTO, ex-Dropbox)** (search-result summary citing [Contrary Research](https://research.contrary.com/reports/persona) and [Visa "Ask a fintech founder"](https://corporate.visa.com/en/sites/visa-perspectives/innovation/ask-a-fintech-founder-persona.html); individual pages not opened).
- **Legal name:** Persona Identities, Inc. ([KYA page footer](https://marketing.withpersona.com/solutions/know-your-agent/)).
- **Funding:** $200M Series D announced April 30 2025 at a $2B valuation, led by Founders Fund and Ribbit Capital, with BOND, Chemistry, Coatue, First Round and Index participating ([Series D post](https://marketing.withpersona.com/blog/series-d)). Series C was $150M at $1.5B in September 2021 ([Biometric Update](https://www.biometricupdate.com/202109/persona-raises-150m-renews-development-efforts-in-digital-id-solutions-and-infrastructures); [fintech.global](https://fintech.global/2025/04/30/identity-verification-firm-persona-hits-2bn-valuation-following-200m-funding-round/)). Total raised roughly $417.5M per third-party aggregators — unverified.
- **Scale:** 300M+ verifications in 2024; revenue and customer count roughly doubled year over year ([Series D post](https://marketing.withpersona.com/blog/series-d)). Headcount ~500–620 and "3,000+ customers" come from third-party directories — unverified.
- **Notable news:**
  - **Feb 2026 controversy:** researchers alleged exposed frontend code and data flows to US agencies in connection with OpenAI identity checks; Discord ended a short age-verification test with Persona after user backlash. These are allegations reported in the press, not established findings ([Malwarebytes](https://www.malwarebytes.com/nl/blog/news/2026/02/age-verification-vendor-persona-left-frontend-exposed), [eWeek](https://www.eweek.com/news/discord-persona-age-verification-backlash/), [MediaPost](https://www.mediapost.com/publications/article/413004/discord-ditches-age-verification-partner-following.html)). Relevance: Persona is now leaning hard on privacy-preserving messaging (Relay) — do not joke about surveillance in the demo, and do show data minimisation.
  - **Candidate Verification** launch for hiring fraud ([PR Newswire](https://www.prnewswire.com/news-releases/persona-launches-candidate-verification-to-stop-hiring-fraud-before-day-one-302711200.html)).
  - **Oct 2026:** $100K academic research grants on identity, fraud and privacy ([Biometric Update](https://www.biometricupdate.com/202610/persona-offers-100k-grants-for-academic-research-on-identity-fraud-and-privacy)).
  - **KYA content run in 2026:** Jan 26 (KYA + dynamic IDV), Mar 25 (Lithic guest post), Apr 21 (AstraSync guest post), Apr 24 (Crossmint guest post), Jul 28 (KYA explainer by Kerwell Liao), plus comments to NIST on AI agent security (title listed on the [KYA page](https://marketing.withpersona.com/solutions/know-your-agent/); the NIST post itself was not opened).

---

## 4. Customers & use cases

- **Named on the KYA page trust bar:** OpenAI, Lithic, Crossmint ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)).
- **Named in the Series D post:** LinkedIn, Block, OpenAI ([Series D post](https://marketing.withpersona.com/blog/series-d)).
- **Named in the Jan 2026 KYA post:** Etsy, GetYourGuide, TaskRabbit, OpenAI ([post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv)).
- **Others via third-party listings (unverified):** Twilio, Square, Udemy, Travelex, Gusto, Instacart, Robinhood, Reddit, Roblox ([Terrapinn sponsor profile](https://www.terrapinn.com/exhibition/identity-week/sponsor-persona.stm), search summaries).
- **Verticals:** fintech, marketplaces, digital health, payments, crypto, government, financial institutions, e-learning, higher education ([KYA page nav](https://marketing.withpersona.com/solutions/know-your-agent/)).

### Agentic-commerce case studies (all guest posts on Persona's blog)

| Partner | What they do with Persona | Source |
|---|---|---|
| **Lithic** (card issuer) | Agents already buy with Lithic-issued cards; Persona does KYC/KYB at onboarding so "there's a verified and legitimate entity behind every account and every agent". Cardholders set where/when/how much an agent may spend; "The agent should act on behalf of a person or business, not instead of one." Notes merchants blocked agents lacking device-fingerprint signals. | [Mar 25 2026](https://marketing.withpersona.com/blog/agentic-commerce-is-happening-now-heres-what-weve-learned) |
| **Crossmint** (agent wallets / virtual cards, lobster.cash) | Persona is the KYC/AML provider; agent transactions needing identity or compliance checks are "automatically routed through Persona". Humans set per-transaction limits, daily caps, merchant allowlists and approval thresholds. | [Apr 24 2026](https://marketing.withpersona.com/blog/whos-behind-agent-security-trust-compliance-agentic-payments) |
| **AstraSync** (KYA infrastructure) | Developer completes Persona gov-ID + liveness (+ KYB) before registering agents; each agent gets a cryptographic identifier tracing to a verified human/legal entity, a dynamic trust score, and a blockchain-anchored audit trail. | [Apr 21 2026](https://marketing.withpersona.com/blog/building-know-your-agent-the-missing-identity-layer-for-agentic-commerce) |

Persona's worked commerce examples: an agent pre-authorised to spend "up to $75 tonight" on dinner, where the merchant confirms recent authorisation and that the user is old enough to order wine; and World Cup tickets where the agent "could check out on its own for some events, but not the World Cup", so the merchant requests a selfie as step-up ([Jan 26 2026 post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv)).

---

## 5. Why they are at this hackathon (inferred)

- **KYA is a 2026 go-to-market push with no self-serve product yet.** The KYA page's only CTA is a sales form ("Ready to build trust into your AI agents?") and the Help Center article ends with "To learn more or request access, let us know" ([KYA page](https://marketing.withpersona.com/solutions/know-your-agent/), [Verifying AI Agents](https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/)). A hackathon gets real agent builders to assemble KYA patterns from the existing primitives, which gives product marketing reference implementations and stories.
- **They sent a Product Marketing Manager, not a developer advocate.** Kerwell L. is both judge and speaker ([Luma page](https://luma.com/personalagenthack)), and Kerwell Liao wrote the definitive KYA explainer ([Jul 28 2026](https://marketing.withpersona.com/blog/what-is-know-your-agent-kya)). Expect narrative and use-case clarity to weigh as much as code depth: a demo that can be retold as "here is how a shopping agent does KYA" is what a PMM can use.
- **The prize is modest and non-cash** ($250 Keychron gift card + swag per member, [event page](https://tokensand.com/personalagenthack)), and the only link they gave organisers is the Help Center, not the API docs ([event brief](../00-event-brief.md)). That signals an awareness/education goal and probably a thin field of entrants for this track — most teams will chase the cash tracks.
- **Developer surface they want exercised:** sandbox Inquiries (hosted/embedded), Relay humanness claims, Signals, and their new AI-facing tooling (MCP server, docs `llms.txt`, docs MCP at `https://docs.withpersona.com/_mcp/server`) ([docs llms.txt header](https://docs.withpersona.com/llms.txt)).
- **Positioning they want reinforced:** identity as the trust layer for agentic commerce, with privacy (post-February 2026) — "verifying the human behind an AI agent" ([Series D post](https://marketing.withpersona.com/blog/series-d)).

---

## 6. Prize track

- **Sponsor tagline (exact):** "Detect agents and verify the people behind them." ([event page](https://tokensand.com/personalagenthack)). Luma variant: "helps businesses detect agents and verify the people behind agents." ([Luma](https://luma.com/personalagenthack))
- **Track (exact):** "Best Use of Persona" ([event page](https://tokensand.com/personalagenthack)).
- **Prize (exact):** "$250 gift cards to Keychron + custom Persona swag for each member of the winning team." ([event page](https://tokensand.com/personalagenthack)). One winning team; no 2nd/3rd listed.
- **Judging criteria:** none published on either page.
- **Link supplied by organisers:** https://help.withpersona.com/ only.

### People

**Kerwell L. — Product Marketing Manager @ Persona (judge and speaker)** ([Luma](https://luma.com/personalagenthack)).

- Almost certainly **Kerwell Liao**: Persona's blog bylines the KYA explainer to "Kerwell Liao", described as a product marketing manager focused on Persona's identity verification solutions who "enjoys watching basketball and exploring the world with his German Shepherd" ([What is KYA, Jul 28 2026](https://marketing.withpersona.com/blog/what-is-know-your-agent-kya)).
- Background per a third-party aggregator (unverified): PMM for Identity at Persona since March 2023; previously Stripe Radar (2021–2022) and YouTube research (2015–2021); 10+ years of user research across Google, YouTube and Stripe ([village.ai profile](https://village.ai/person/kerwell-l-MzI3NDkwNDcx)). If accurate, he has a fraud-product (Stripe Radar) and UX-research lens: he will care about friction vs assurance and about whether the user experience of verification is believable.
- What he has written about agents — the KYA explainer's key ideas:
  - Definition: "Know Your Agent, or KYA, is a general framework for understanding AI agents, the people behind them, and the risk that agents may pose."
  - Every KYA programme balances **assurance, friction and privacy**.
  - Four use cases; **agentic commerce is #1**, where the relying party is the merchant or PSP and the questions are: did the right human authorise the agent, is the purchase "within the user-defined authorization limits", and is there "an audit trail and documentation for chargeback representment".
  - Higher value needs higher assurance: "a $1,000 purchase versus a $10 purchase".
  - High-risk actions may require "reauthorizing or reverifying the person before allowing the agent to continue".
  - Caveat he will appreciate hearing back: "You shouldn't assume an agent is safe just because it's verified."
- Other Persona voices on agents (not attending, useful to quote): Ross Freiman-Mendel, head of product growth, author of the Jan 2026 post and speaker on the on-demand webinar "Know your agent: A new problem for old defenses" with Lithic, Glenbrook and Fintech Takes ([webinar page](https://marketing.withpersona.com/events/know-your-agent-a-new-problem-for-old-defenses/)).

No other Persona attendees are listed.

---

## 7. What they expect to see used (ranked by likely impact on this sponsor)

1. **The full KYA lifecycle, built from Inquiries + Accounts:** verify the human (Gov ID + Selfie Inquiry) → link the agent to that person's Account/reference ID in your product → scope the agent's authority to what was verified → **reverify on a high-risk action**. This is literally the "Verify the person before you link the agent" procedure and the Onboarding/Reverification lifecycle ([Verifying AI Agents](https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/), [KYA page](https://marketing.withpersona.com/solutions/know-your-agent/)). Highest impact, fully doable in sandbox.
2. **Step-up / "differential risk" tied to the action:** light check for cheap purchases, selfie re-verification above a threshold, explicit human approval recorded for hard-to-undo actions (the four-rung ladder) ([Verifying AI Agents](https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/), [Jan 2026 post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv)).
3. **Auditability:** store inquiry ID, status, timestamps, template, and the human's approval alongside each agent action. Persona lists auditability as one of four capabilities of good KYA and "an audit trail and documentation for chargeback representment" as a core commerce question ([Jan 2026 post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv), [What is KYA](https://marketing.withpersona.com/blog/what-is-know-your-agent-kya)).
4. **Relay (Live Human Presence, or Age 21+ Verified for an age-restricted purchase):** their newest, most differentiated, privacy-first product; "Your integration is your audit trail" is their own phrasing ([Integrating Relay](https://help.withpersona.com/articles/cOtN25GojXD3Ou6Bz5xADy/)). Very impressive **if** it is enabled in your sandbox (it is plan-gated — see risks).
5. **Webhooks with signature verification** (`inquiry.approved`, `inquiry.declined`, `inquiry.failed`) as the source of truth instead of client callbacks — shows you read the docs ([Accessing Inquiry status](https://docs.withpersona.com/2026-09-29/accessing-inquiry-status)).
6. **Keeping PII away from the model** and using Device Handoff for in-chat verification ([Verify users directly in chat](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).
7. **Agent detection via Signals / Sentinel** (Bot Score, behaviour threat level). On-message for "detect agents", but `signals` on the Inquiry API "are gated behind a feature flag at the organization level" ([Inquiry Signals](https://docs.withpersona.com/2026-09-29/inquiry-signals)) and Sentinel needs enablement ([Sentinel overview](https://help.withpersona.com/articles/OzIPtG0LQsMTVqp22x0t4Q/)). Ask at the booth; do not plan around it.
8. **Persona MCP server** so your agent (or your coding agent) can create inquiries and read results ([MCP article](https://help.withpersona.com/articles/3Ev9FHB3kXENDUwbGg20KX/)). Novel and agent-native, but it cuts against Persona's own advice that the model should not start verifications or see verification data — use it for the operator/audit side, not the shopper-facing agent.

---

## 8. Quickstart

### Signup and access

- **Self-serve sandbox: yes.** "Sign up to get immediate access to a sandbox API key and start evaluating the API with sample data" ([API Keys](https://docs.withpersona.com/2026-09-29/api-keys)). Signup at https://withpersona.com/signup; the quickstart says "Register with your business email and create your organization" ([API quickstart](https://docs.withpersona.com/2026-09-29/api-quickstart-tutorial)). Whether a personal Gmail address is rejected is **unverified** — sign up tonight, not at 11:00.
- **Trial terms:** free 60-day trial, no credit card, Essential Plan features, "up to 50 services", sandbox-only by default; production needs a business verification review ([Plans overview](https://help.withpersona.com/articles/6oZbzp7jb7AWGClF5vpY3K/)). Paid Essential starts at $250/month; the Startup Program gives 500 free services/month for a year ([Plans overview](https://help.withpersona.com/articles/6oZbzp7jb7AWGClF5vpY3K/), [Startup Program](https://help.withpersona.com/articles/1XNnqukfZY9VamF2e7jkuJ/)). Whether sandbox inquiries count against the 50 is unverified; docs say sandbox incurs no usage charges ([Environments](https://docs.withpersona.com/2026-09-29/environments)).
- **Set-up steps in the Dashboard:** switch to Sandbox → Solutions > Explore → Verify Individuals → **Add solution** "KYC" (creates an inquiry template plus two Workflows that auto-approve passing inquiries) → Inquiries > Templates, copy the `itmpl_...` ID → API > API Keys, copy the `persona_sandbox_...` key ([Add a solution](https://help.withpersona.com/articles/67J7FurQtIgwxkWWvUropu/), [Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api), [API quickstart](https://docs.withpersona.com/2026-09-29/api-quickstart-tutorial)).

### Auth

HTTP Bearer over HTTPS: `Authorization: Bearer $API_KEY` ([Authentication](https://docs.withpersona.com/2026-09-29/authentication)). Sandbox keys start with `persona_sandbox`, production with `persona_production` ([API quickstart](https://docs.withpersona.com/2026-09-29/api-quickstart-tutorial)). Optional `Persona-Version` header; current default is `2026-09-29`. Base URL `https://api.withpersona.com/api/v1`. Keep the key server-side.

### A. Create an inquiry (server)

From the official Hosted Flow tutorial ([source](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api)):

```bash
curl https://api.withpersona.com/api/v1/inquiries \
-X POST \
-H "Authorization: Bearer YOUR_API_KEY" \
-H "Content-Type: application/json" \
-H "Persona-Version: 2025-10-27" \
-d '{
    "data": {
        "attributes": {
            "inquiry-template-id": "itmpl_XXXXXXXXXXXXX",
            "reference-id": "usr_ABC123",
            "fields": {
                "name_first": "Alexander",
                "name_last": "Sample",
                "birthdate": "1977-08-31"
            }
        }
    }
}'
```

The response's `data.id` is the inquiry ID (`inq_...`). Note: the API reference marks `data.attributes.reference-id` as deprecated in favour of `meta.auto-create-account-reference-id` ([Create an Inquiry](https://docs.withpersona.com/2026-09-29/api-reference/inquiries/create-an-inquiry)), which is the form used in the chat guide ([source](https://docs.withpersona.com/2026-09-29/chat-identity-verification)):

```bash
curl -X POST https://api.withpersona.com/api/v1/inquiries \
  -H "Authorization: Bearer $PERSONA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "attributes": {
        "inquiry-template-id": "itmpl_..."
      }
    },
    "meta": {
      "auto-create-account-reference-id": "your-user-id"
    }
  }'
```

`meta` also accepts `auto-create-one-time-link` and `auto-create-inquiry-session` booleans ([Create an Inquiry](https://docs.withpersona.com/2026-09-29/api-reference/inquiries/create-an-inquiry)).

### B. Show the flow

**Hosted (zero front-end code):** send the user to

```
https://inquiry.withpersona.com/verify?inquiry-id=inq_XXXXXXXXXXXXX
```

([Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api)). Add `redirect-uri` to send them back with an `inquiry-id` GET parameter; if a redirect allowlist is set, your domain must be on it ([Hosted Flow parameters](https://docs.withpersona.com/2026-09-29/hosted-flow-parameters)).

**Embedded, with a pre-created inquiry** ([chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification)):

```javascript
import Persona from 'persona';

const client = new Persona.Client({
  inquiryId: '<inquiry ID from your backend, starting with inq_>',
  onReady: () => client.open(),
  onComplete: ({ inquiryId }) => {
    // Update the UI, then ask your backend for the result.
    notifyBackendOfCompletion(inquiryId);
  },
  onCancel: ({ inquiryId, sessionToken }) => showResumeOption(inquiryId, sessionToken),
  onError: (error) => showFallbackLink(error)
});
```

**Embedded, no backend at all (fastest possible demo)** — script tag `https://cdn.withpersona.com/dist/persona-vX.Y.Z.js` (latest in the changelog at time of research: 5.10.0) or the `persona` npm package ([Embedded tutorial](https://docs.withpersona.com/2026-09-29/tutorial-embedded-flow-inquiry-template), [SDK changelog](https://docs.withpersona.com/2026-09-29/embedded-flow-changelog)):

```javascript
const client = new Persona.Client({
    templateId: "itmpl_XXXXXXXXXXXXX",
    environmentId: "env_XXXXXXXXXXXXX",
    referenceId: userId,
    fields: userFields,
    onReady: () => client.open(),
    onComplete: ({ inquiryId, status, fields }) => {
        // send inquiryId to your backend
        client.destroy();
    },
});
```

Gotcha: omit `environmentId` and it defaults to **production** ([Embedded tutorial](https://docs.withpersona.com/2026-09-29/tutorial-embedded-flow-inquiry-template)).

### C. Read the result

Polling (explicitly "fine for a demo" per the [chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification)):

```bash
curl https://api.withpersona.com/api/v1/inquiries/inq_XXXXXXXXXXXXX \
-X GET \
-H "Authorization: Bearer YOUR_API_KEY" \
-H "Content-Type: application/json" \
-H "Persona-Version: 2025-10-27"
```

Read `data.attributes.status` (expect `approved` with the KYC solution, because its Workflows auto-approve passing inquiries), `data.attributes.reference-id`, `data.attributes.fields` ([Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api)).

Webhook (Dashboard > Webhooks > Create webhook; URL must be public HTTPS, e.g. ngrok; enable `inquiry.started`, `inquiry.completed`, `inquiry.approved`, `inquiry.declined`, `inquiry.failed`). Signature check adapted from the official Node sample — header `persona-signature: t=<unix>,v1=<hex>`, HMAC-SHA256 of `timestamp + '.' + rawBody` with the webhook secret ([Webhook quickstart](https://docs.withpersona.com/2026-09-29/quickstart-webhooks)):

```javascript
const crypto = require('crypto');
// app.use(express.json({ verify: (req, res, buf) => { req.rawBody = buf.toString('utf8'); } }));

function verifyPersonaSignature(rawBody, signatureHeader, webhookSecret) {
  const signaturePairs = signatureHeader.split(' ');
  const timestamp = signaturePairs[0].split(',')[0].split('=')[1];
  const signatures = signaturePairs
    .map(pair => { const m = pair.match(/v1=([^,]+)/); return m ? m[1] : null; })
    .filter(Boolean);
  const expected = crypto.createHmac('sha256', webhookSecret)
    .update(timestamp + '.' + rawBody).digest('hex');
  return signatures.some(sig => {
    try { return crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(sig)); }
    catch (e) { return false; }
  });
}

app.post('/webhook', (req, res) => {
  if (!verifyPersonaSignature(req.rawBody, req.headers['persona-signature'], process.env.PERSONA_WEBHOOK_SECRET))
    return res.status(401).end();
  const data = req.body.data;
  // data.attributes.name            -> e.g. "inquiry.approved"
  // data.attributes.payload.data.id -> "inq_..."
  res.status(200).json({ success: true });
});
```

Persona retries if it gets no success within 5 seconds, up to 7 more times with backoff ([Webhook quickstart](https://docs.withpersona.com/2026-09-29/quickstart-webhooks)).

### D. Drive the sandbox without a human (demo safety net)

`POST /api/v1/inquiries/{inquiry-id}/perform-simulate-actions` on a sandbox inquiry; available action types include `start_inquiry`, `complete_inquiry`, `fail_inquiry`, `approve_inquiry`, `decline_inquiry`, `mark_for_review_inquiry`, `expire_inquiry`, `create_passed_verification`, `create_failed_verification`; each fires the matching webhook events ([Integration testing](https://docs.withpersona.com/2026-09-29/integration-testing), [API ref](https://docs.withpersona.com/2026-09-29/api-reference/inquiries/perform-simulate-actions)):

```json
{
  "meta": {
    "simulate-actions": [
      { "type": "start_inquiry" },
      { "type": "complete_inquiry" },
      { "type": "approve_inquiry" }
    ]
  }
}
```

Use this to demo the *declined* step-up path deterministically (`decline_inquiry`), and as a fallback if camera/venue Wi-Fi fails.

### E. Relay (only if enabled in your sandbox)

Server, Node (`npm install @persona/sdk-node`), from the official quickstart ([Relay Server SDK quickstart](https://docs.withpersona.com/2026-09-29/relay-sdk-quickstart)):

```typescript
import { Persona } from '@persona/sdk-node';
const persona = new Persona({ apiKey: '<your_api_key>' });

const { relayToken, relaySecret, relaySessionAccessToken } =
  await persona.relays.create({
    claimType: "live_human_presence",
    encryptionKeyJwk: null, // pass null to opt out of encryption
  });

const { privacyPassToken } = await persona.relays.issuePrivacyPass({
  claimType: "live_human_presence",
});

// after the client reports completion:
const { claimPayload, tokenConsumed } = await persona.relays.generateClaim({
  relayToken, relaySecret, privacyPassToken,
});
const claim = JSON.parse(claimPayload); // { claim_type, claim_result: 'passed'|'failed', methodology }
```

Client (`npm install @persona/relay`) ([Relay Embedded Widget](https://docs.withpersona.com/2026-09-29/relay-widget-usage)):

```javascript
import Relay from "@persona/relay";
const relay = new Relay("#relay-container", {
  accessToken: "<relay-session-access-token from your server>",
  onComplete: () => { /* ask your server to retrieve the claim result */ },
  onExpire: () => { /* session expires after 30 minutes */ },
  onError: (error) => console.error(error),
});
```

The docs example writes the claim type as the string `"live_human_presence"`; the schema page says to obtain the "Claim Type ID" from Dashboard → Relay ([Relay schemas](https://docs.withpersona.com/2026-09-29/relay-schemas)) — confirm which value your account expects.

### Sandbox behaviour, limits, gotchas

- **No real verification in sandbox.** A "Pass verifications" toggle at the bottom of the flow forces pass or fail; do not enter real personal data ([Environments](https://docs.withpersona.com/2026-09-29/environments), [Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api)).
- **Sandbox overwrites the name to "Alexander J Sample"** and much response data is null/demo ([Sandbox vs Production](https://help.withpersona.com/articles/6I2kGhfPvSuUjYq4z6tpmB/)). Do not build logic that depends on extracted name/DOB; say "sandbox / simulated verification" in the demo (the organisers' FAQ also asks you to show whether actions are real, simulated or test).
- **Sandbox Relay claims always return `passed`**, "even if the verification flow was never started or completed", and Relay "must be enabled for your sandbox environment, which depends on your Persona plan" ([Relay sandbox](https://docs.withpersona.com/2026-09-29/relay-sandbox)).
- **`onComplete` is not a source of truth** and may never fire (e.g. user finishes on phone); decide on the backend from webhook or API ([chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification), [Accessing Inquiry status](https://docs.withpersona.com/2026-09-29/accessing-inquiry-status)).
- **Use decisioning statuses** (`approved` / `declined` / `needs_review`), not just `completed` / `failed`, when the template has Workflows ([Accessing Inquiry status](https://docs.withpersona.com/2026-09-29/accessing-inquiry-status)).
- **Allowed Domains / redirect allowlist:** embedded flow only runs on allow-listed domains if a list is configured; add `localhost` and your deploy domain, each subdomain separately ([chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).
- **Nested iframes break camera access**; prefer Hosted Flow in a new tab or Device Handoff QR over embedding inside a chat widget iframe ([chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).
- **Inquiries expire 24 hours after creation by default**; handoff links after 1 hour ([Hosted Flow tutorial](https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api), [chat guide](https://docs.withpersona.com/2026-09-29/chat-identity-verification)).
- **Rate limit:** production default is 300 requests/minute per environment, with separate creation quotas; sandbox figure not stated — check Dashboard > API > Rate Limits. Poll at ~1–2 s, not in a tight loop ([Rate limiting](https://docs.withpersona.com/2026-09-29/rate-limiting)).
- **Webhook URL must be public HTTPS** ([Webhook quickstart](https://docs.withpersona.com/2026-09-29/quickstart-webhooks)).
- **Inquiry Signals on the API need a feature flag** enabled by Persona ([Inquiry Signals](https://docs.withpersona.com/2026-09-29/inquiry-signals)).
- **Docs for coding agents:** `https://docs.withpersona.com/llms.txt`, append `.md` to any docs/help page, docs MCP at `https://docs.withpersona.com/_mcp/server` ([docs llms.txt](https://docs.withpersona.com/llms.txt)).

---

## 9. Past hackathons this company sponsored

- **Public hackathon sponsorships with a Persona prize track: not found.** Searches for "Best Use of Persona", Persona + Devpost/Luma/hackathon winners, and KYA hackathons returned no prior sponsored events, tracks or winning projects. This event (tokensand "Personal Agents Hackathon", Nov 20 2026) is the only one located ([Luma](https://luma.com/personalagenthack)).
- **Internal only:** Persona's People/Talent team ran a non-technical internal AI hackathon (blog dated June 29 2026): 10 projects, five winners; one project let candidates verify their identity with Persona to see their pipeline stage; "Best Execution" went to a New Hire Onboarding Hub. Not a sponsored external event ([blog](https://withpersona.com/blog/lessons-from-running-a-successful-non-technical-ai-hackathon/), read via search summary only).
- **Winning projects from previous Persona tracks:** not found — none to report, and none invented here.

### Patterns in what wins (inferred, since there is no winner history)

With no precedent and no published rubric, the best proxy is Persona's own published checklist for agentic commerce. A project scores if it visibly demonstrates:

1. **Origin** — the request traces to a known, verified human ([Jan 2026 post](https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv)).
2. **Consent with scope** — the agent is bound to that human for specific actions under specific limits.
3. **Differential risk** — step-up verification when the action gets riskier.
4. **Auditability** — a record of what the agent requested, what the human approved, and what happened, privacy-first.

Plus the **assurance / friction / privacy** trade-off from the judge's own article ([What is KYA](https://marketing.withpersona.com/blog/what-is-know-your-agent-kya)): show that cheap purchases are frictionless and only risky ones interrupt the human. A first-time sponsor with a PMM judge is also likely to reward the team that uses the product most "by the book" and explains it most clearly.

---

## 10. Fit with "shopping agent with an audit trail"

Persona's framing fits the challenge unusually well: the agent's "reliable context" includes *who authorised it and with what limits*, the "meaningful action" is a purchase gated by that authority, and the Persona verification record is a first-class audit-trail entry.

| # | Idea | What you build | Persona pieces | Time in a 5.5h build |
|---|---|---|---|---|
| 1 | **Verify the human once, before linking the agent** (shallow) | On first run, backend creates an Inquiry with the user's ID as reference ID; user completes the Hosted Flow in a new tab; backend polls until `approved`; only then is the shopping agent "linked" and allowed to act. Store `inq_...`, status, timestamps. | Create Inquiry, Hosted Flow link, GET Inquiry, KYC solution template | ~30–45 min (assuming signup done beforehand) |
| 2 | **Verification record as an audit-trail entry** | Every agent decision/purchase row in the audit log carries `verified_by: persona`, inquiry ID, account reference ID, status, `completed-at`, environment (`sandbox`), and what the human approved. Render it in the trail UI next to retrieval evidence (e.g. Senso `content_id`s). Mirrors "an audit trail and documentation for chargeback representment". | Inquiry attributes; optionally webhook event IDs (`evt_...`) as tamper-evident receipts | +20–30 min on top of #1 |
| 3 | **Delegated purchase authority bound to the verification ("mandate")** | After approval, the user sets scope: per-order cap, daily cap, merchant/category allowlist, expiry. Your backend issues a signed mandate (e.g. a JWT or HMAC'd JSON) that embeds the Persona inquiry ID + account reference ID. The agent must present it at the checkout tool; the tool rejects anything out of scope. This is Persona's "consent with scope" and "link the agent ... and limit what it can do to what you verified" — the mandate itself is your construct, not a Persona feature; say so. | Accounts/reference IDs, Inquiry result | ~45–60 min |
| 4 | **Step-up reverification above a spend threshold** (the money shot) | Under $X the agent checks out silently. Over $X, or for a restricted category (e.g. alcohol → age), the checkout tool pauses, creates a *second* Inquiry (selfie-only template if you have time, otherwise the same one) under the same reference ID, shows a QR/link, and resumes only on `approved`; `declined` blocks the purchase and both outcomes are logged. Use `perform-simulate-actions` or the "Pass verifications" toggle to demo approve and decline live. Echoes Persona's World Cup tickets and "$1,000 vs $10" examples. | Second Inquiry, webhooks (`inquiry.approved` / `inquiry.declined`) with signature check, simulate actions, Device Handoff | ~60–90 min incl. webhook + ngrok |
| 5 | **Privacy-preserving humanness / age claim via Relay, plus agent-vs-human signals** (deep, conditional) | Replace the step-up with a Relay `live_human_presence` (or Age 21+ Verified) claim so your app never touches PII; store claim type, result, method and time as the audit entry. Optionally show Bot Score / behaviour threat level on the merchant side to "detect the agent". | Relay Server SDK + widget; Inquiry Signals / Sentinel | ~60–120 min, **only if** Relay/Signals are enabled for your sandbox — confirm at the booth before starting |

**Recommended scope:** #1 + #2 + #4 (with #3 as a thin signed-mandate object) is about 2–2.5 hours of one person's time, entirely within documented sandbox behaviour, and demonstrates all four of Persona's stated KYA capabilities. Treat #5 as a stretch only after the booth confirms access.

**Demo script hints for this judge:** say "Know Your Agent" out loud; show the cheap purchase going through with zero friction and the expensive one triggering reverification (assurance vs friction); show that the LLM only ever sees `verified` / `not_verified` (privacy); label the verification as sandbox/simulated; end on the audit trail row with the Persona inquiry ID.

---

## 11. Risks and open questions for the booth

**Risks**

- **Sandbox proves nothing about identity.** Verification is simulated and names are overwritten to "Alexander J Sample" ([Sandbox vs Production](https://help.withpersona.com/articles/6I2kGhfPvSuUjYq4z6tpmB/)). A judge from Persona knows this; be upfront, and make the *orchestration* (gating, step-up, audit) the substance.
- **No agent-specific API.** Anything you call an "agent credential" or "mandate" is your own construct layered on Persona. Don't claim Persona issued it.
- **Signup friction:** "business email" wording, onboarding flow, possible email verification. Do it before the event.
- **Feature gating:** Relay in sandbox is plan-dependent; Inquiry Signals need a feature flag; Sentinel needs enablement; Verify with Persona is closed beta.
- **Camera / iframe / venue Wi-Fi:** embedded flow inside a chat iframe can lose camera access; have the Hosted Flow link and `perform-simulate-actions` as fallbacks.
- **Webhook plumbing eats time** (ngrok, HTTPS, signature). Polling is officially acceptable for a demo; add webhooks only if time permits.
- **Trial cap of 50 services** — unclear whether sandbox inquiries count; avoid creating inquiries in loops.
- **Small prize, single winner, PMM judge:** worth doing only if the integration also strengthens the main project's audit-trail story (it does).
- **Reputational sensitivity:** Persona had a privacy controversy in Feb 2026; lean into data minimisation, not surveillance-flavoured features.

**Questions to ask Kerwell / the Persona table**

1. What are the judging criteria for "Best Use of Persona"? Is depth of API use or the KYA story weighted more?
2. Is there a hackathon org/template or promo to enable **Relay** (Live Human Presence, Age 21+ Verified) and **Inquiry Signals / Bot Score** in our sandbox today?
3. Does a personal email work for signup, and do sandbox inquiries count against the 50-service trial cap? What is the sandbox rate limit?
4. Is there a recommended selfie-only / reverification template for step-up, or should we reuse the KYC solution template?
5. Is there any Persona-issued artefact we can bind to an agent (token, credential, Account relation), or is "link the agent in your own product" the intended pattern?
6. Is there a "detect agents" capability usable without a verification flow in sandbox (Sentinel), and what does it return for a headless browser agent?
7. Does Persona integrate today with any agentic-payments protocol (Stripe ACP, Visa Intelligent Commerce, Mastercard Agent Pay, AP2), or is that roadmap?
8. Is using the Persona MCP server from the agent viewed as a plus, given the docs say the model should not start verifications?
9. Does the Claim Type for Relay get passed as the string `live_human_presence` or as a Dashboard ID?

**Could not verify**

- Past Persona-sponsored hackathons, tracks, or winners (none found).
- Kerwell L.'s full surname and career history from a first-party source (blog byline says "Kerwell Liao"; Stripe/YouTube history is from a third-party aggregator only).
- Any talk title for Persona's speaker slot.
- Total funding, headcount, and customer count (third-party figures only).
- Contents of Persona's NIST comments post (title seen, post not read).
- Whether personal email addresses are accepted at signup.

---

## 12. Sources

**Event**
- https://tokensand.com/personalagenthack
- https://luma.com/personalagenthack
- `/Users/thilakshekharshriyan/Projects/PersonalHack/research/00-event-brief.md`

**Persona marketing / blog** (read via the `marketing.withpersona.com` mirror; canonical `withpersona.com` URLs return 403 to automated fetches)
- https://marketing.withpersona.com/solutions/know-your-agent/ (canonical: https://withpersona.com/solutions/know-your-agent/)
- https://marketing.withpersona.com/blog/what-is-know-your-agent-kya
- https://marketing.withpersona.com/blog/safe-agentic-commerce-starts-with-kya-and-dynamic-idv
- https://marketing.withpersona.com/blog/building-know-your-agent-the-missing-identity-layer-for-agentic-commerce
- https://marketing.withpersona.com/blog/agentic-commerce-is-happening-now-heres-what-weve-learned
- https://marketing.withpersona.com/blog/whos-behind-agent-security-trust-compliance-agentic-payments
- https://marketing.withpersona.com/blog/series-d
- https://marketing.withpersona.com/events/know-your-agent-a-new-problem-for-old-defenses/
- https://withpersona.com/blog/lessons-from-running-a-successful-non-technical-ai-hackathon/ (search summary only)
- https://withpersona.com/lp/know-your-agent/ (search summary only)

**Persona docs** (append `.md` for Markdown)
- https://docs.withpersona.com/llms.txt and https://docs.withpersona.com/2026-09-29/llms.txt
- https://docs.withpersona.com/2026-09-29/getting-started
- https://docs.withpersona.com/2026-09-29/environments
- https://docs.withpersona.com/2026-09-29/authentication
- https://docs.withpersona.com/2026-09-29/api-keys
- https://docs.withpersona.com/2026-09-29/api-quickstart-tutorial
- https://docs.withpersona.com/2026-09-29/api-reference/inquiries/create-an-inquiry
- https://docs.withpersona.com/2026-09-29/api-reference/inquiries/perform-simulate-actions
- https://docs.withpersona.com/2026-09-29/tutorial-hosted-flow-unique-api
- https://docs.withpersona.com/2026-09-29/hosted-flow-parameters
- https://docs.withpersona.com/2026-09-29/tutorial-embedded-flow-inquiry-template
- https://docs.withpersona.com/2026-09-29/embedded-flow-changelog
- https://docs.withpersona.com/2026-09-29/chat-identity-verification
- https://docs.withpersona.com/2026-09-29/accessing-inquiry-status
- https://docs.withpersona.com/2026-09-29/inquiry-signals
- https://docs.withpersona.com/2026-09-29/integration-testing
- https://docs.withpersona.com/2026-09-29/quickstart-webhooks
- https://docs.withpersona.com/2026-09-29/rate-limiting
- https://docs.withpersona.com/2026-09-29/reference-ids
- https://docs.withpersona.com/2026-09-29/relay
- https://docs.withpersona.com/2026-09-29/relay-getting-started
- https://docs.withpersona.com/2026-09-29/relay-sandbox
- https://docs.withpersona.com/2026-09-29/relay-sdk-quickstart
- https://docs.withpersona.com/2026-09-29/relay-widget-usage
- https://docs.withpersona.com/2026-09-29/relay-schemas

**Persona Help Center** (append `.md` for Markdown; index at https://help.withpersona.com/llms.txt)
- https://help.withpersona.com/articles/CmxigxTOZVhlr4bn7xSX9S/ (Verifying AI Agents)
- https://help.withpersona.com/articles/3Ev9FHB3kXENDUwbGg20KX/ (Connect AI agents to Persona with MCP)
- https://help.withpersona.com/articles/OzIPtG0LQsMTVqp22x0t4Q/ (Sentinel overview)
- https://help.withpersona.com/articles/3aYYkI9NMDsyVqx5V3IT3G/ (Signals overview)
- https://help.withpersona.com/articles/cOtN25GojXD3Ou6Bz5xADy/ (Integrating Relay)
- https://help.withpersona.com/articles/ytHNjFfek8K22wv85UPgVS/ (Relay: Live Human Presence)
- https://help.withpersona.com/articles/4Yl6a7VEH2ScPfAHDif0Ts/ (Verify with Persona)
- https://help.withpersona.com/articles/6I2kGhfPvSuUjYq4z6tpmB/ (Sandbox vs Production)
- https://help.withpersona.com/articles/6oZbzp7jb7AWGClF5vpY3K/ (Plans overview)
- https://help.withpersona.com/articles/1XNnqukfZY9VamF2e7jkuJ/ (Startup Program)
- https://help.withpersona.com/articles/67J7FurQtIgwxkWWvUropu/ (Add a solution)

**Third-party**
- https://village.ai/person/kerwell-l-MzI3NDkwNDcx (Kerwell L. profile; unverified aggregator, search summary only)
- https://fintech.global/2025/04/30/identity-verification-firm-persona-hits-2bn-valuation-following-200m-funding-round/
- https://www.biometricupdate.com/202109/persona-raises-150m-renews-development-efforts-in-digital-id-solutions-and-infrastructures
- https://www.biometricupdate.com/202610/persona-offers-100k-grants-for-academic-research-on-identity-fraud-and-privacy
- https://research.contrary.com/reports/persona (search summary only)
- https://corporate.visa.com/en/sites/visa-perspectives/innovation/ask-a-fintech-founder-persona.html (search summary only)
- https://komo.ai/directory/persona (search summary only)
- https://www.terrapinn.com/exhibition/identity-week/sponsor-persona.stm (search summary only)
- https://www.prnewswire.com/news-releases/persona-launches-candidate-verification-to-stop-hiring-fraud-before-day-one-302711200.html (search summary only)
- https://www.malwarebytes.com/nl/blog/news/2026/02/age-verification-vendor-persona-left-frontend-exposed (search summary only)
- https://www.eweek.com/news/discord-persona-age-verification-backlash/ (search summary only)
- https://www.mediapost.com/publications/article/413004/discord-ditches-age-verification-partner-following.html (search summary only)
- https://www.silicon.co.uk/press-release/vouched-launches-know-your-agent-verification-to-bring-trust-and-identity-to-the-next-generation-of-ai-agents (search summary only)
- https://idtechwire.com/incode-launches-know-your-agent-binding-ai-agents-to-a-face-verified-person-and-a-spending-mandate/ (search summary only)
- https://www.prove.com/blog/agentic-commerce-identity-crisis-kya (search summary only)
- https://www.paz.ai/blog/agentic-commerce-trust-stack-identity-layer (search summary only)
