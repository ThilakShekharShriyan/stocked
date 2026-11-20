# 03 — Shopping-agent landscape (late 2026)

> Researched 2026-11-19. This market changes monthly and much of the coverage is vendor or trade-blog material, so each claim carries its source and I flag where only secondary sources exist. **Inference** marks my own reading. Sponsor products are out of scope here (other files cover them); building blocks below are the non-sponsor parts.

## 1. The short version

- The big players spent 2025 launching "buy inside the chat". By 2026 the pattern shifted to **"discover in AI, buy on the merchant's site"**: OpenAI withdrew ChatGPT Instant Checkout in March 2026 after weak uptake ([ecomcrew](https://www.ecomcrew.com/openai-pulls-back-on-chatgpt-instant-checkout/), [analysis](https://www.digitalapplied.com/blog/ai-agentic-commerce-discover-in-ai-buy-on-site-2026)).
- Protocols have consolidated around **UCP** (Google, Shopify and a long list of retailers and payment firms) for catalog, cart, checkout and orders, and **AP2** (now at the FIDO Alliance) for signed payment authorisation ([ucp.dev](https://ucp.dev/), [ap2-protocol.org](https://ap2-protocol.org/)).
- **Consumer trust is the unsolved problem.** Surveys this year: three-quarters would stop or pause using an agent that bought the wrong thing without checking (attributed to a Synchrony / Oxford Economics survey in search summaries; I did not open the original, so treat as unverified); 6 in 10 UK adults would drop a shopping agent after one mistake; 46% of US shoppers who bought on an AI recommendation returned an item; half think the AI platform should refund agent mistakes, while platform terms put the liability on the user ([ppc.land on ACI/YouGov](https://ppc.land/60-of-uk-adults-would-drop-an-ai-shopping-agent-after-one-mistake-aci-finds/), [Retail Customer Experience on Blue Yonder](https://www.retailcustomerexperience.com/news/ai-data-issue-driving-product-returns/), [Riskified](https://www.riskified.com/press/agentic-commerce-pulse-consumer-survey-november-2026/), [Forbes](https://www.forbes.com/sites/jordanmckee/2026/10/01/agentic-commerce-has-a-consumer-trust-problem/)).
- **The gap a hack can fill:** incumbents have cryptographic proof that a user *authorised a payment*. None shows a shopper, in plain language, *why the agent chose this item, from which facts, and what it ruled out* — or carries that record through to a return or dispute. That is the challenge statement.

## 2. The incumbents

| Player | What it does today | Where it is weak |
|---|---|---|
| **ChatGPT shopping (OpenAI)** | Product research and recommendations in chat; Instant Checkout launched Sep 29, 2025 with Stripe on the Agentic Commerce Protocol (ACP), then was withdrawn around March 2026 in favour of merchant apps and hand-off to the merchant's checkout. Reported reasons: about 8% of US adult users tried it in the first month; Etsy saw little volume; Walmart reportedly saw in-chat checkout convert about 3x worse than click-through ([ecomcrew](https://www.ecomcrew.com/openai-pulls-back-on-chatgpt-instant-checkout/), [swell](https://www.swell.is/content/agentic-commerce-protocol-explained), [digitalapplied](https://www.digitalapplied.com/blog/ai-agentic-commerce-discover-in-ai-buy-on-site-2026)). All secondary sources; I found no primary OpenAI post. The Atlas browser's agent mode could shop on sites, but OpenAI's own page now says Atlas is deprecated ([OpenAI](https://openai.com/index/introducing-chatgpt-atlas/)). | Stops at recommendation. No persistent mandate, no price watching, no post-purchase. Sources behind a recommendation are thin. |
| **Google (AI Mode, Gemini, UCP, AP2)** | UCP announced Jan 11, 2026; checkout in AI Mode and Gemini with Etsy and Wayfair, then a Buy button in Search, per a secondary timeline ([thomas-wiegold](https://thomas-wiegold.com/blog/agentic-commerce-ucp/)). AP2 handles signed authorisation. | The proof is cryptographic and aimed at banks and merchants, not readable by a shopper. US-first. Merchant must be onboarded. |
| **Amazon (Rufus → "Alexa for Shopping")** | Renamed May 2026. Price history up to a year; price alerts; **Auto Buy** at a target price, checked every 30 minutes, 24 hours to cancel before shipping, active for six months; scheduled actions; **Buy for Me** purchases from other retailers' sites on the shopper's behalf ([Modern Retail](https://www.modernretail.co/technology/marketplace-briefing-amazons-shopping-bot-rufus-can-now-automatically-buy-products-for-you-when-prices-drop/), [ppc.land](https://ppc.land/amazons-rufus-ai-assistant-gains-memory-price-tracking-and-auto-buying/), [letsdatascience](https://letsdatascience.com/blog/amazon-retires-rufus-alexa-for-shopping-agent)). | A walled garden with a conflict of interest: Amazon chooses what to show. Some retailers objected to Buy for Me. Auto Buy's only rule is price. No explanation of why this listing over another. Amazon also sued Perplexity to keep an outside agent off its site (injunction granted March 2026, reportedly overturned on appeal; case continuing) ([Payments Dive](https://www.paymentsdive.com/news/amazon-sues-perplexity-ai-shopping-agents/804923/), [The Next Web](https://thenextweb.com/news/amazon-loses-perplexity-comet-ai-shopping-ruling)). |
| **Perplexity (Instant Buy, Comet)** | Instant Buy with PayPal for US users since Nov 2025; merchant fulfils; merchants on several commerce platforms ([Perplexity help](https://www.perplexity.ai/help-center/en/articles/12932923-instant-buy-buy-with-paypal)). Comet browser agent shops on sites. | Limited merchant set; browser-agent access is legally contested; no long-running tasks or post-purchase. |
| **Shopify (Shop MCP, catalog servers, UCP)** | **Shop MCP** at `https://mcp.shop.com/` — search, product detail, cart and checkout tools with no API key; signed-in buyers can let the agent complete checkout, read saved addresses and track orders; "spend requests" issue a single-purchase card for non-Shopify stores. One checkout per merchant ([Shopify docs](https://shopify.dev/docs/agents/shop-mcp), [overview](https://shopify.dev/docs/agents)). | Shopify merchants only; multi-store baskets need several checkouts; agent-completed checkout outside Shop needs Shopify's permission. |
| **Stripe** | Agentic Commerce Suite for sellers; **Shared Payment Tokens** — scoped to one seller, bounded by amount and time, observable through their lifecycle; Link Agent Wallet for agents; machine payments (MPP, x402); UCP support ([Stripe blog](https://stripe.com/blog/agentic-commerce-suite), [docs](https://docs.stripe.com/agentic-commerce)). | Plumbing, not a shopper-facing product. The token says "up to $X at merchant Y until time T", not why. |
| **Visa Intelligent Commerce** | Tokenised agent credentials and the Trusted Agent Protocol for verifying a legitimate agent at a merchant; "Intelligent Commerce Connect" as one integration across four protocols, per secondary coverage ([eMarketer](https://www.emarketer.com/content/visa-s-agentic-commerce-hub-positions-network-agentic-middle-man), [TechInformed](https://techinformed.com/visa-opens-one-integration-for-ai-agent-payments/)). | Network-level; needs issuer and merchant adoption; nothing consumer-visible. |
| **Mastercard Agent Pay** | "Agentic Tokens" built on its tokenisation; live agent transactions reported in several markets in 2026; a machine-payments variant launched in June, per secondary coverage ([comparison](https://eco.com/support/en/articles/15192003-mastercard-agent-pay-vs-visa-trusted-agent-2026-compared)). | Same as Visa. Dates here come from explainer sites — treat as unverified. |
| **PayPal** | Agent Ready (accept payments from AI assistants) and Store Sync (catalog distribution), launched Oct 2025; an agent wallet ([PayPal developer](https://developer.paypal.com/agent-ready/overview), [press release](https://www.prnewswire.com/news-releases/paypal-launches-agentic-commerce-services-to-power-ai-driven-shopping-302596548.html)). | Access by request; buyer protection for agent purchases is asserted more than explained. |
| **Klarna** | New AI assistant for shopping, returns, bookings and admin, on WhatsApp and iMessage; finds the best price, buys, pay-later; early-user rollout with a waitlist ([Klarna investor news](https://investors.klarna.com/News--Events/news/news-details/2026/From-returns-to-restaurant-bookings-Klarnas-AI-assistant-finds-the-best-product--price-buys-it-for-you-and-you-dont-pay-until-it-arrives-2026-AsJ__j3bx3/default.aspx), [The Paypers](https://thepaypers.com/payments/news/klarna-launches-ai-assistant-for-shopping-returns-and-bookings)). | Still a chat interface; revenue model for steered purchases undisclosed (affiliate conflict); very new. This is the closest incumbent to "returns included". |
| **Browser and computer-use agents** | Drive real retail sites through the page. | Slow and brittle; blocked by bot defences; terms-of-service and legal risk (see Amazon v. Perplexity); exposed to prompt injection from page content; the "audit trail" is screenshots. |

### Weaknesses that recur across all of them

1. **No reasons.** None shows the evidence behind a choice at the level of "this claim came from this document, this version".
2. **Wrong-item risk.** Size, fit, variant and compatibility errors drive returns ([Blue Yonder survey](https://www.retailcustomerexperience.com/news/ai-data-issue-driving-product-returns/)).
3. **Liability gap.** Shoppers expect the platform to fix agent mistakes; terms say otherwise ([Riskified](https://www.riskified.com/press/agentic-commerce-pulse-consumer-survey-november-2026/)).
4. **Delegation is blunt.** A spend cap and an expiry, at best. No "ask me if it is over $80, never substitute brands, only stores with 30-day returns".
5. **Post-purchase is thin.** Returns, refunds, warranty claims, price-drop refunds, failed deliveries: Klarna is starting; others stop at the receipt.
6. **Single-retailer baskets.** Each checkout is one merchant; nobody optimises a basket across stores with shipping and return terms included.
7. **Stuck in chat.** Almost all are a chat box. The organizers called this out directly.
8. **Conflict of interest.** Retailer-owned agents and affiliate-funded agents choose what to show.

## 3. What "audit trail" means in agentic commerce

Two different things share the name. A strong project shows both and is clear about which is which.

### 3a. Authorisation trail — "the user really allowed this payment"

**AP2 (Agent Payments Protocol), v0.2, donated to the FIDO Alliance** ([ap2-protocol.org](https://ap2-protocol.org/), [repo](https://github.com/google-agentic-commerce/AP2)):

- Built on **Verifiable Digital Credentials**: tamper-evident, cryptographically signed objects.
- Current docs describe two mandate types, each with an **open** and a **closed** stage:
  - **Checkout Mandate** (shared with the merchant) — open: the user's constraints and goals before a cart exists; closed: authorisation for one specific, finalised checkout.
  - **Payment Mandate** (shared with credential provider, networks and processor) — open: budget and allowed payment instruments; closed: a specific amount bound to that checkout.
- The mandates are **chained** to give "a non-repudiable, cryptographic audit trail for every transaction" for dispute resolution, in both **human-present** and **human-not-present** flows.
- Earlier (2025) write-ups call these the **Intent Mandate**, **Cart Mandate** and **Payment Mandate** ([Descope explainer](https://www.descope.com/learn/post/ap2), [Vellum](https://www.vellum.ai/md/blog/googles-ap2-a-new-protocol-for-ai-agent-payments)). Same idea, older names. Either vocabulary will be recognised; "intent → cart → payment" is the easier one to say on stage.
- I could not open the field-level schema (the spec URL returned 404); the repo holds JSON schemas and Pydantic models.

**Related:** Stripe Shared Payment Tokens (seller-scoped, amount- and time-bounded) ([Stripe](https://stripe.com/blog/agentic-commerce-suite)); Visa's Trusted Agent Protocol and Mastercard's Agentic Tokens (agent identity and tokenised credentials at the network level); UCP identity linking over OAuth so the agent never holds the shopper's password ([ucp.dev](https://ucp.dev/)).

### 3b. Decision trail — "why the agent did this"

No standard exists. This is where the challenge points. A complete record per decision:

| Layer | What to record |
|---|---|
| Intent | The shopper's request in their words, plus the structured constraints derived from it (budget, must-haves, exclusions, deadline) |
| Mandate | What the agent may do alone, what needs approval, spend cap, expiry — signed or at least hashed and timestamped |
| Context provenance | Every fact used: source, document or record ID, **version**, the retrieved text, retrieval score and rank, time fetched. Senso's result fields map straight onto this (see event brief) |
| Candidates | What was considered and **why each loser was rejected** |
| Checks | Each rule evaluated (budget, return window, compatibility, stock, policy) with pass or fail and the evidence |
| Uncertainty | What the agent did not know, and whether it asked, assumed, or stopped |
| Approval | Who approved, what exactly they saw, when |
| Action | The call made, idempotency key, response, and a clear **real / test mode / simulated** label |
| Outcome | Order or payment ID, receipt, and later events (shipped, returned, refunded) linked to the same record |
| Integrity | Append-only log; each entry hash-chained to the previous one so tampering is detectable |

### 3c. What a compelling audit-trail interface looks like (inference)

- **A receipt, not a log.** The primary artefact is a "decision receipt": what was bought, for how much, under which mandate, with three to five plain-language reasons, each linked to its evidence.
- **Click any claim to see its source.** "30-day returns" opens the exact policy passage, with document version and retrieval time.
- **A timeline with states**, not a chat transcript: Understood → Searched → Compared → Checked → Asked → Bought → Tracking. Each step expandable.
- **The road not taken.** A short "considered and rejected" list with a reason per item. This is what shoppers cannot get from any incumbent.
- **Mandate shown as a gauge**: spent against the cap, time left, which rules applied.
- **Honest labels**: a persistent badge for TEST MODE or SIMULATED on anything that is not real money.
- **Gaps made visible**: "I could not confirm the warranty length — I asked you" rather than a confident guess. The organizers' own FAQ asks for this.
- **Verify button**: recompute the hash chain live and show it passes; edit one record and show it fail.
- **Two audiences**: a shopper view in plain language and an "auditor" view with IDs, hashes and raw payloads.
- **Exportable**: one click to a shareable link or PDF, usable as evidence in a return or dispute.

## 4. Building blocks for a 5.5-hour hack

Goal: a "meaningful action" that is real or credibly simulated, and labelled as such. Ranked by reliability under time pressure. Nothing below was tested hands-on — check access tonight.

| Option | What you get | Effort | Risk | Notes |
|---|---|---|---|---|
| **Stripe test mode / sandbox** | A real payment API with fake money: PaymentIntents or Checkout Sessions, test card `4242 4242 4242 4242`, refunds, webhooks, a dashboard you can show on screen | Low | Low | The most defensible "action": the charge and the refund appear in Stripe's own dashboard. Use idempotency keys. Docs: [testing](https://docs.stripe.com/testing), [agents](https://docs.stripe.com/agents). Shared Payment Tokens and Link Agent Wallet exist but availability varies by integration ([docs](https://docs.stripe.com/agentic-commerce)) — do not depend on preview access. |
| **Your own mock retailer(s)** | Two or three small storefronts with product JSON, stock, prices, policy pages and an order endpoint | Low–medium | Low | Lets you seed edge cases (price changes mid-run, out of stock, a promo exclusion, different return windows) and reset between demos. Fits the organizers' FAQ, which suggests a fictional retailer's catalog and policies. Give it a real-looking order admin page so the action visibly lands somewhere. |
| **Shop MCP (Shopify)** | Live search across real Shopify merchants, product detail, cart and checkout creation with no API key; hand-off to the merchant's checkout page | Low | Medium | Real catalog data with zero setup makes the "reliable context" part real. Requires an MCP client on spec version 2026-07-28; buying needs a signed-in buyer, so stop at "checkout created" and label it ([docs](https://shopify.dev/docs/agents/shop-mcp)). |
| **Shopify development store + Storefront API** | Your own store with test products, a real cart and checkout URL, test payment gateway, orders in a real admin | Medium | Medium | Needs a Partner account and store set up **before** the event (account setup is preparation, not project code). Strong on-screen proof: the order appears in Shopify admin. |
| **UCP samples and playground** | Reference merchant and agent implementations of catalog, cart, checkout, orders | Medium | Medium | Useful to speak the standard's language; samples at [github.com/Universal-Commerce-Protocol/samples](https://github.com/Universal-Commerce-Protocol/samples), playground linked from [ucp.dev](https://ucp.dev/). |
| **AP2 sample code** | Runnable human-present card flow with mandates; Python, Go, Android, web client | Medium–high | Medium–high | Needs Python 3.11+, `uv` and a Google API key; one script starts the scenario ([repo](https://github.com/google-agentic-commerce/AP2)). Cheaper alternative: implement AP2-*shaped* mandates yourself — JSON objects signed with an Ed25519 key in the browser or server — and say plainly that they follow AP2's structure and are not a conformant implementation. |
| **Browser automation on a real retailer** | The agent drives an actual site | High | High | Bot blocking, logins, CAPTCHAs, slow, breaks on stage, terms-of-service issues, and never completes a real purchase safely. Use only against your own mock store, where it demonstrates "works on any site" without the risk. |
| **Non-payment real actions** | Email a return request, create a calendar hold for delivery, file a price-match claim, cancel a subscription in a sandbox, send an SMS for approval | Low | Low | A real side effect outside your app is persuasive and cheap. Approval by text message is a good "beyond chat" moment. |

**Recommended combination (inference):** a mock retailer you control (for context, edge cases and resets) + Stripe test mode (for an action with an external, inspectable record) + self-signed AP2-shaped mandates and a hash-chained decision log (for the audit trail) + one real side effect such as an approval text or a return email. Optionally pull live products from Shop MCP for realism. Label every action real, test or simulated — the organizers ask for this in their FAQ.

**Prepare before 11:00 (not project code):** Stripe account and test keys; any Shopify partner or dev store; phone number or email sandbox for approvals; API keys for the sponsor tools you intend to use; confirm your MCP client version; fixture content (fictional products and policies) drafted as documents.

## 5. Ten differentiated project ideas

All avoid travel booking, all put the audit trail on screen as a designed surface, and all end in an action. "Why judges care" is inference from the organizer and judge research in files 01 and 02.

| # | Idea | What it does | Meaningful action | Why judges would care |
|---|---|---|---|---|
| 1 | **Standing-order agent with a signed mandate** ("buy it when it is right") | Shopper sets rules once — item, price ceiling, acceptable sellers, minimum return window, never substitute. Agent watches and buys only when every rule passes, re-verifying price and stock at the moment of purchase. | Test-mode purchase triggered without a human click, inside a visible mandate | Hits the Autonomy criterion head-on, and beats Amazon Auto Buy, whose only rule is price. The mandate gauge and rule checks are the audit trail. |
| 2 | **Returns and refunds agent** ("the receipt fights for you") | Reads past orders, finds items still inside a return window or eligible for a price-drop refund, checks the retailer's policy text, files the claim and tracks it. | Refund issued in Stripe test mode plus a return request sent | Post-purchase is where incumbents are weakest and where shoppers lose money. Policy citations per claim are a natural trail. Not a chat window. |
| 3 | **Wrong-item preventer** (compatibility and fit verifier) | Before buying a part, cartridge, charger, filter or garment, checks it against what the shopper already owns (model numbers, past sizes by brand) and blocks or swaps mismatches. | Corrected cart checked out; mismatched item visibly refused | Targets the top cause of AI-driven returns with evidence per check. Retail-data realism for the Gap judge; process-checking for the Plato and Firedrill judges. |
| 4 | **Household replenishment with a shared ledger** | Family or flatmates share a budget and rules; the agent restocks staples across two or three stores, asks the right person for approval by text when a rule requires it, and keeps a ledger everyone can read. | Multi-store order placed after a text-message approval | Multi-user permissions echo the last overall winner (a family assistant that "only takes actions the family allows"). Approval by text is beyond chat. |
| 5 | **Urgent home-repair purchase** ("the water heater died") | A homeowner with no expertise describes the failure. Agent identifies the compatible replacement from the old unit's label, checks local stock, delivery date, installation requirements and warranty, and orders the unit and the parts that go with it. | Order placed with a delivery slot held | A non-tech, non-SF customer with a costly, time-critical problem. Speaks directly to the Broccoli founders' market and to "think beyond SF". |
| 6 | **Benefit-card shopper** (FSA/HSA or grocery-assistance rules) | Builds a basket that complies with programme rules, shows the rule passage that makes each item eligible or not, splits payment between benefit and personal card, and stores proof for a later audit. | Split payment in test mode with an eligibility record per line | Real compliance stakes, an under-served user, and a ledger-style trail an Intuit engineer will recognise. Past overall winners skewed to healthcare and compliance. |
| 7 | **Buying for someone else** (carer or adult child purchasing for a parent) | Delegated buyer acts within limits set by the person being bought for; identity of the human behind the agent is confirmed before sensitive or high-value purchases; every purchase is explained to both people in plain language. | Purchase completed only after identity check and consent | Clear human story, accessibility angle for the Meta judge, natural fit for an identity sponsor, and delegation limits are the weakest part of incumbent products. |
| 8 | **Total-cost-of-ownership buyer** | For a printer, appliance or subscription-tied device, computes multi-year cost (consumables, energy, warranty, return terms) from sourced figures and buys the cheapest to own, not the cheapest to buy. | Purchase of the non-obvious winner | A counter-intuitive result with a number attached ("costs $40 more today, $310 less over three years"), every figure sourced. Shows reasoning incumbents do not show. |
| 9 | **Agent you can crash** (reliability-first checkout) | Deliberately kill the agent after payment, change the store's price or stock, restart: it reads its own write-ahead log, does not double-pay, notices what changed and finishes correctly. Run a naive agent beside it that pays twice. | One correct charge where the naive agent makes two | A near-identical demo won a track at the Sep 25 event with one of tomorrow's judges on the panel. The log that makes recovery possible is the audit trail. |
| 10 | **Shopping agent with a scorecard** (test harness as the product) | The agent ships with a resettable simulated retailer and a suite of scenarios — bait-and-switch price, fake reviews, prompt injection hidden in a product description, expired promo, "do not buy" cases — and reports pass/fail with the trail for each. | Purchases or refusals across the suite, live | Two judges run agent-simulation companies. Showing that the agent refuses to buy when it should, and proving it across scenarios, is rare in hackathon demos. |

**Combining them (inference):** ideas 1, 3 and 9 share one architecture (mandate + rule checks + write-ahead decision log) and could be one project: a standing order that verifies compatibility and survives a crash. Idea 10 is a demo technique that strengthens any of the others. Ideas 5, 6 and 7 are the strongest user stories; pair one of them with the architecture of 1 or 3.

**Avoid:** flight, hotel or trip booking (dismissed by the organizers); a chat box as the main interface; a "price comparison" that ends in a list of links (a shopping project with that shape left the Sep 25 event without an award — see file 01); driving a real retailer's site with a browser on stage; any action whose real/test/simulated status is unclear.

## 6. Could not verify

- A primary OpenAI statement on Instant Checkout's withdrawal (trade coverage only; dates vary between early and late March 2026).
- Exact UCP rollout dates, the Walmart "3x worse" conversion figure, and Visa and Mastercard launch dates (secondary sources).
- AP2 field-level schema for v0.2 (spec page returned 404; mandate names taken from the documentation home page).
- Current status of Amazon v. Perplexity after the reported appeal ruling.
- Whether Stripe Shared Payment Tokens and Link Agent Wallet are usable in a new sandbox account without preview access.
- Rate limits and any sandbox guidance for Shop MCP (not stated in its docs).
- None of the building blocks were run hands-on.

## Sources

- Protocols: https://ap2-protocol.org/ · https://github.com/google-agentic-commerce/AP2 · https://ucp.dev/ · https://github.com/Universal-Commerce-Protocol/samples · https://www.descope.com/learn/post/ap2 · https://www.vellum.ai/md/blog/googles-ap2-a-new-protocol-for-ai-agent-payments · https://thomas-wiegold.com/blog/agentic-commerce-ucp/ · https://www.digitalapplied.com/blog/agentic-commerce-standards-ucp-acp-ap2-2026-merchant-guide
- OpenAI: https://www.ecomcrew.com/openai-pulls-back-on-chatgpt-instant-checkout/ · https://www.swell.is/content/agentic-commerce-protocol-explained · https://www.digitalapplied.com/blog/ai-agentic-commerce-discover-in-ai-buy-on-site-2026 · https://openai.com/index/introducing-chatgpt-atlas/
- Amazon: https://www.modernretail.co/technology/marketplace-briefing-amazons-shopping-bot-rufus-can-now-automatically-buy-products-for-you-when-prices-drop/ · https://ppc.land/amazons-rufus-ai-assistant-gains-memory-price-tracking-and-auto-buying/ · https://letsdatascience.com/blog/amazon-retires-rufus-alexa-for-shopping-agent · https://www.paymentsdive.com/news/amazon-sues-perplexity-ai-shopping-agents/804923/ · https://thenextweb.com/news/amazon-loses-perplexity-comet-ai-shopping-ruling
- Perplexity, PayPal, Klarna: https://www.perplexity.ai/help-center/en/articles/12932923-instant-buy-buy-with-paypal · https://developer.paypal.com/agent-ready/overview · https://www.prnewswire.com/news-releases/paypal-launches-agentic-commerce-services-to-power-ai-driven-shopping-302596548.html · https://investors.klarna.com/News--Events/news/news-details/2026/From-returns-to-restaurant-bookings-Klarnas-AI-assistant-finds-the-best-product--price-buys-it-for-you-and-you-dont-pay-until-it-arrives-2026-AsJ__j3bx3/default.aspx · https://thepaypers.com/payments/news/klarna-launches-ai-assistant-for-shopping-returns-and-bookings
- Shopify and Stripe: https://shopify.dev/docs/agents · https://shopify.dev/docs/agents/shop-mcp · https://stripe.com/blog/agentic-commerce-suite · https://docs.stripe.com/agentic-commerce · https://docs.stripe.com/testing · https://docs.stripe.com/agents
- Card networks: https://www.emarketer.com/content/visa-s-agentic-commerce-hub-positions-network-agentic-middle-man · https://techinformed.com/visa-opens-one-integration-for-ai-agent-payments/ · https://eco.com/support/en/articles/15192003-mastercard-agent-pay-vs-visa-trusted-agent-2026-compared
- Trust surveys: https://ppc.land/60-of-uk-adults-would-drop-an-ai-shopping-agent-after-one-mistake-aci-finds/ · https://www.retailcustomerexperience.com/news/ai-data-issue-driving-product-returns/ · https://www.riskified.com/press/agentic-commerce-pulse-consumer-survey-november-2026/ · https://www.forbes.com/sites/jordanmckee/2026/10/01/agentic-commerce-has-a-consumer-trust-problem/ (headline only; page not readable) · https://www.smartcustomer.com/resources/ai-shopping-survey-2026
- Event context: files `00-event-brief.md`, `01-organizer-and-past-events.md`, `02-judges.md` in this folder
