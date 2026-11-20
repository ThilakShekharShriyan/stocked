# Personal Agents Hackathon — Event Brief (source text)

> Raw facts as published by the organizers. Everything else in `research/` builds on this.
> Event page: https://tokensand.com/personalagenthack

## Basics

- **Date:** Friday, November 20, 2026 · San Francisco, CA
- **Hack window:** 11:00 AM PT → **4:30 PM PT submission deadline** (5.5 hours)
- **Team:** 2–4 people, one project per team
- **Build during the event.**

## Challenge statement

**Build a shopping agent with an audit trail.**

Build a personal agent that helps someone make a purchase or complete a shopping task. The agent should:

1. use **reliable context** to make decisions,
2. take a **meaningful action**,
3. **show the information and steps** behind what it did.

"Pick a real problem, build a working experience, and show us the result."

Organizer framing: "Personal agents can do far more than their demos show. Booking a flight or planning a trip? That's stuff you'd happily do yourself. The web is going agent-native, but most agents are still stuck inside chat apps. We think a personal agent should be its own category. Think beyond the problems in SF. Build a personal agent that can compete with the current ones." — i.e. **beyond a chat window**.

> **Update, Nov 20 (event Discord):** asked whether the shopping agent is mandatory or just a sponsor track, Andy Tran (Lead Organizer) replied: "Not mandatory, just a suggestion". Any personal agent can compete. Senso's own track wording still says "shopping agent".

## What to submit

- GitHub repository reviewers can access
- Short demo video with a shareable link
- What you built and the tools you used
- Team names and contact emails
- Optional but welcome: working website, project screenshot

## Schedule (PT)

| Time | What |
|---|---|
| 9:30 AM | Doors open + opening remarks |
| 11:00 AM | Kickoff & hack |
| 1:30 PM | Lunch |
| 4:30 PM | Project submission deadline |
| 5:00 PM | Finalist demos + judging |
| 7:00 PM | Awards + closing |

## Sponsors & prizes

| Sponsor | Tagline | Prize track | Prizes |
|---|---|---|---|
| Senso.ai | Context Layer | Best Use of Senso: shopping agent with trusted context; show how the agent uses context, the action it takes, the trail behind decisions | 1st 10k credits · 2nd 5k · 3rd 1k |
| Superlinked | Task-specific inference for AI agents | (a) Best Personal Agent Harness Using SIE Serverless; (b) Best Technical Build for Personal Agents with SIE | each: $500 cash + $500 SIE Serverless credits |
| Amplitude + Statsig | Agent analytics, feature flags, experimentation | Best Use of Amplitude or Statsig | 1st $1,000 · 2nd $500 |
| Guild AI | Infrastructure for building, evaluating, deploying AI agents | Most Interesting Use of Agents | 1st 100M tokens · 2nd 50M · 3rd 25M |
| Airbyte | Open-source data integration, 700+ sources; "The Context Layer for AI Agents" | (unnamed — Airbyte Context docs linked) | $1,000 / $500 / $250 Visa gift cards |
| Arcee AI | Open-weight foundation models | Most Impressive Use of the Pareto Frontier | 1st $500 credits + $250 merch card · 2nd $250 credits + $100 card · 3rd $100 card |
| Cognee | Open-source memory for AI agents | Best use of cognee | 1st $500 cash + $500 credits · 2nd $250 cash + $500 credits · 3rd $500 credits |
| Firedrill | Simulation platform for AI agents | Best Use of Firedrill | 1st $500 Visa + 3mo Starter + 40k credits · 2nd $300 + 3mo + 20k · 3rd $200 + 3mo + 14k |
| SingleStore | Real-time data for AI agents | Best Use of SingleStoreDB | $1,000 / $750 / $500 gift cards |
| Persona | Detect agents and verify the people behind them | Best Use of Persona | $250 Keychron gift cards + swag per member |

## Sponsor links supplied

- Senso: https://docs.senso.ai/docs/overview · https://docs.senso.ai/llms.txt · https://docs.senso.ai/specs/sdk-api.yaml · https://docs.senso.ai/api-keys · https://docs.senso.ai/docs/senso-cli · https://docs.senso.ai/docs/cli-reference
- Superlinked: https://superlinked.com/serverless · https://superlinked.com/docs · https://github.com/superlinked/cookbook
- Amplitude: https://amplitude.com/docs · Statsig: https://docs.statsig.com/welcome
- Guild AI: https://docs.guild.ai/ · https://app.guild.ai/chat
- Airbyte: https://docs.airbyte.com/platform/context-layer
- Arcee AI: https://docs.arcee.ai/
- Cognee: https://docs.cognee.ai/ · https://docs.cognee.ai/cookbooks/personalized-email · https://github.com/topoteretes/cognee
- Firedrill: https://firedrill.run/hacker-docs · https://docs.firedrill.run/ · https://firedrill.run/
- SingleStore: https://docs.singlestore.com/cloud/getting-started-with-singlestore-helios/ · https://docs.singlestore.com/cloud/developer-resources/connect-with-application-development-tools/connect-with-python/ · https://docs.singlestore.com/cloud/developer-resources/functional-extensions/working-with-vector-data/
- Persona: https://help.withpersona.com/

## Senso hackathon FAQ (from the organizers' Google Doc)

- **Context Layer:** a knowledge base the agent can search and use when making decisions. E.g. add a fictional retailer's product details, return policy and shipping info, then retrieve relevant context while helping someone shop.
- **Docs for coding agents:** every docs page has a `.md` version; index at `/llms.txt`; OpenAPI at `/specs/sdk-api.yaml`. `llms.txt` tells agents to install the CLI and run `senso setup` first.
- **MCP vs API vs CLI:** MCP to connect an AI app (Claude, ChatGPT, Codex, Cursor) to the KB. API when your app calls Senso at runtime. CLI from a terminal. "A shopping agent you ship will usually use the API, and you can use MCP to load and inspect your context while you build."
- **Content:** fictional product descriptions, retailer policies, return instructions, FAQs. Text, Markdown, supported uploads. Only content you have permission to share.
- **MCP permissions:** tick "Read documents" and "Create and edit documents". Leave delete OFF (deleting a folder deletes everything in it). Leave AI visibility permissions off.
- **Audit trail fields:** each search result includes `content_id`, `kb_node_id`, `version_id`, `chunk_text`, `score`, `rank` and how it was retrieved. **Log these with every decision the agent makes.**
- **Doc not found?** Documents must finish processing before they are searchable; check org and read permission.
- **Missing context:** make it visible. Ask the shopper, suggest a next step, or stop before acting. Never present an unsupported claim as fact.
- **Purchases:** Senso supplies context only. Your app does the cart/reservation/payment. **Show clearly whether the action is real, simulated or test mode.**

## Judges

| Judge | Role |
|---|---|
| Riya Dashoriya | Engineering Manager @ Meta |
| Lingxi Li | Software Engineer @ Cursor |
| Robert F. | Co-founder & CEO @ Plato |
| Harshal Vaidya | Staff Software Engineer @ Intuit |
| Rebecca Abu Sharkh | Solutions Engineering @ Amplitude |
| Daniel Svonava | Founder & CEO @ Superlinked |
| Rahul Parundekar | DevRel @ Superlinked |
| Anneketh Vij | AI Research Scientist @ Arcee AI |
| Sam Fraser | Product @ Arcee AI |
| Veljko Kovac | Head of FDE @ Cognee |
| Corbett Waddingham | Head of Developer Relations @ Guild.ai |
| Richelle Herrli | Senior Developer Advocate @ Airbyte |
| Newton Asare | Co-founder and CEO @ Firedrill |
| AJ Jain | CEO @ Broccoli |
| Aditya Ketkar | CTO @ Broccoli |
| Kerwell L. | Product Marketing Manager @ Persona |
| Amanda Kwong | Solutions Engineer @ SingleStore |
| Ryan Sarginson | Solutions Engineer @ SingleStore |
| Michael Moyo | CEO & Co-Founder @ Ocular AI (YC W24) |
| Mogana Kumaran Sivaraman | Sr Staff Software Engineer @ GAP Inc |

## Speakers

Sid Nayak (Principal SE, Amplitude) · Jackson Yang (SE, Amplitude) · Daniel Svonava (Superlinked) · Veljko Kovac (Cognee) · Sam Fraser (Arcee AI) · Richelle Herrli (Airbyte) · Corbett Waddingham (Guild.ai) · Kiran Das (Co-founder & CTO, Firedrill) · Kerwell L. (Persona) · Amanda Kwong (SingleStore)
