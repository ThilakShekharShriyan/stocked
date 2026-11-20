# Arcee AI — sponsor research

> Researched 2026-11-19 for the Personal Agents Hackathon (Nov 20 2026, SF). Challenge: "build a shopping agent with an audit trail".
> Every claim has a source URL. Where something could not be confirmed it is marked **not found** or **unverified**.

---

## 1. TL;DR

- **What it is.** A US open-weight model lab. It trains its own Trinity models (Nano 6B, Mini 26B, Large 400B sparse MoE) and runs the **Arcee Open Models API**, an OpenAI-compatible endpoint that serves Trinity-Large-Thinking next to other open models (DeepSeek, GLM, Kimi, Inkling) at prices from about $0.14 to $15 per 1M tokens ([blog, Aug 13 2026](https://www.arcee.ai/blog/open-models-api-beta), [docs pricing](https://docs.arcee.ai/get-started/pricing.md)).
- **What the prize wants.** The track is "Most Impressive Use of the Pareto Frontier" with no published criteria ([event page](https://tokensand.com/personalagenthack)). **The phrase "Pareto frontier" does not appear anywhere in Arcee's docs, website, blog, model cards, platform app or the nac repo** (I searched all of them). The meaning below is my inference: pick, per agent step, the cheapest/fastest open model that is still good enough, and prove it with cost, latency and quality numbers.
- **Why that inference is reasonable.** Arcee's own pitch is frontier-level agents at a fraction of the cost ("#2 on PinchBench ... roughly 96% cheaper" than Opus 4.6, [blog](https://www.arcee.ai/blog/trinity-large-thinking)), "no single model is best for every workload" ([blog](https://www.arcee.ai/blog/open-models-api-beta)), and their older Conductor router routed "to the most appropriate and cost-effective" model ([docs](https://docs.arcee.ai/arcee-conductor/introduction-to-arcee-conductor.md)).
- **Fastest qualifying integration (about 30 min).** Get a key at [platform.arcee.ai](https://platform.arcee.ai/), point the OpenAI SDK at `https://api.arcee.ai/api/v1`, call `GET /api/v1/models` to see what is live, and run the agent's tool-calling loop on an Arcee-hosted model ([quick start](https://docs.arcee.ai/get-started/quick-start.md)).
- **Best way to win.** Build a small per-step model router over the Arcee catalog, log model, tokens, dollars and latency for every step into the audit trail, and show a cost-vs-quality scatter plot with the frontier drawn and your chosen models sitting on it. Ask the booth at 9:30 AM what they mean by "Pareto frontier" before committing.

---

## 2. What they do

### Products today

| Product | What it is | Status | Source |
|---|---|---|---|
| **Trinity model family** | Open-weight sparse mixture-of-experts LLMs trained from scratch in the US. Nano (6B total, 1B active), Mini (26B, 3B active), Large (400B, 13B active). Built for tool calling, multi-turn agents, structured output. | Weights on Hugging Face; Trinity-Large-Thinking on the API | [arcee.ai/trinity](https://www.arcee.ai/trinity), [docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md) |
| **Arcee Open Models API / Arcee Platform** | OpenAI-compatible inference API. Since Aug 13 2026 it serves Trinity plus third-party open models. Pay per token from a wallet. | Live (beta at launch) | [blog](https://www.arcee.ai/blog/open-models-api-beta), [quick start](https://docs.arcee.ai/get-started/quick-start.md) |
| **nac** | Open-source (Apache-2.0) agent harness for long-running work. An orchestrator that can only launch threads; workers return "episodes" (structured summaries). Recommended login is Arcee. | Live, active (last push Nov 2026) | [blog](https://www.arcee.ai/blog/nac), [GitHub](https://github.com/arcee-ai/nac) |
| **Genesis-Science-1** | Trillion-parameter-class open science model built with the US Department of Energy, on next-gen Trinity. | Announced Jul 22 2026, release "later this year" | [blog](https://www.arcee.ai/blog/genesis-science-1) |
| **MergeKit, DistillKit** | Open-source model merging and distillation toolkits (MergeKit about 7.5k GitHub stars). | Maintained | [GitHub org](https://github.com/arcee-ai) |
| **Arcee Conductor** (model router), **Arcee Orchestra** (agent workflows) | Older products. Still documented, but `conductor.arcee.ai`, `models.arcee.ai` and `orchestra.arcee.ai` did not respond from my network on 2026-11-19. Treat as retired unless the booth says otherwise. | **Unverified / likely retired** | [Conductor docs](https://docs.arcee.ai/arcee-conductor/introduction-to-arcee-conductor.md) |
| **Virtuoso, Maestro, Coder, Caller, Blitz** | Earlier fine-tunes of Qwen/Mistral. Docs call them "previously released, and now retired". | Retired | [docs: Post Trains](https://docs.arcee.ai/other/post-trains.md) |

### How it works technically

- Trinity models are sparse MoE: Large has 256 experts with 4 active per token, so roughly 13B of 400B parameters run per token. Arcee says this makes inference "roughly 2-3x faster for the same hardware" than peers in its weight class ([blog](https://www.arcee.ai/blog/trinity-large), [docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-preview.md)).
- Trinity-Large-Thinking is a reasoning model. It emits `<think>` blocks that the API returns as `reasoning_content`. Arcee says these **must be passed back in the message history** for multi-turn tool use to work ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md), [reasoning traces](https://docs.arcee.ai/capabilities/reasoning-traces.md)).
- Published benchmarks for Trinity-Large-Thinking: Tau2-Airline 88.0, Tau2-Telecom 94.7, PinchBench 91.9 (Opus 4.6: 93.3), SWE-bench Verified 63.2 (Opus 4.6: 75.6) ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md)).
- Licence: all Trinity models moved from Apache 2.0 to OpenMDW-1.1 on May 29 2026; commercial use stays free ([blog](https://www.arcee.ai/blog/trinity-is-moving-to-openmdw-1-1)).

### What "the Pareto frontier" most likely means

**Exact phrase in Arcee's own material: not found.** Searched: `docs.arcee.ai/llms-full.txt`, arcee.ai home, /trinity, /blog index and 11 posts, /research, /press, the platform web app bundle, the Hugging Face cards for Trinity-Large-Thinking and Trinity-Mini, the nac repo, and web search. The only place the phrase appears is the prize title on the [event page](https://tokensand.com/personalagenthack).

The general meaning is standard: a model is on the price-performance Pareto frontier if no other model is both cheaper and better ([arXiv 2511.23455](https://arxiv.org/pdf/2511.23455)). Arcee's language maps onto it closely:

- "It scores #2 on PinchBench ... just behind Opus-4.6 while landing at $0.90 per million output tokens on our API, roughly 96% cheaper." ([blog](https://www.arcee.ai/blog/trinity-large-thinking))
- "keeping quality high without making the economics absurd" and "reliable, cheap, high-quality agents" ([same post](https://www.arcee.ai/blog/trinity-large-thinking)).
- "no single model is best for every workload, user, or organization", and Arcee wants to "learn which models users prefer for a given task, and more importantly, why" ([blog](https://www.arcee.ai/blog/open-models-api-beta)).
- "frontier-scale AI does not require frontier-scale waste" ([Series B post](https://www.arcee.ai/blog/arcee-ai-raises-series-b-to-build-the-future-of-american-open-models)).
- Trinity Mini/Nano: "an outstanding price-to-performance ratio" ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-mini-26b.md)).

**What a project would need to show to be "on the frontier" (inferred):**

1. More than one model in use, each chosen for a reason (cheap fast model for easy steps, a stronger model only where it matters).
2. Numbers: per-step tokens, dollars and latency, and a quality measure (task success on a small eval set, or a judge score).
3. A visible frontier: a cost-vs-quality plot of the candidate models or routing policies, with the chosen configuration on the curve and a baseline (everything on the most expensive model) clearly dominated.
4. Ideally the routing decision itself is recorded and explained, which is also exactly what the "audit trail" challenge asks for.

### Positioning and competitors

- Positioning: "Building Open Intelligence", American open-weight models that organisations can "inspect, fine-tune, and deploy" ([arcee.ai](https://www.arcee.ai/)). Strong "US alternative to Chinese open models" angle ([Genesis post](https://www.arcee.ai/blog/genesis-science-1)).
- Competitors named by Arcee or its CEO: DeepSeek, Qwen, Kimi, MiniMax, GLM as the leading open labs ([blog](https://www.arcee.ai/blog/genesis-science-1)); CEO said Arcee is "chasing Z.ai's GLM Flash" ([TNW](https://thenextweb.com/news/arcee-ai-series-b-1bn-valuation-open-weight-trinity)). On the API side it competes with open-model hosts such as OpenRouter, Together and Fireworks (my inference).

---

## 3. Company facts

| Fact | Detail | Source |
|---|---|---|
| Founded | 2023 | [TNW](https://thenextweb.com/news/arcee-ai-series-b-1bn-valuation-open-weight-trinity) |
| Founders | Mark McQuade (CEO, ex-Hugging Face), Brian Benedict, Jacob Solawetz | [SiliconANGLE](https://siliconangle.com/2026/09/16/open-weight-model-developer-arcee-ai-reaches-1b-plus-valuation-with-undisclosed-series-b-funding/), [TNW](https://thenextweb.com/news/arcee-ai-series-b-1bn-valuation-open-weight-trinity) |
| CTO | Lucas Atkins (CTO and head of research) | [TechCrunch](https://techcrunch.com/2026/04/07/i-cant-help-rooting-for-tiny-open-source-ai-model-maker-arcee/) |
| HQ | San Francisco (press release dateline). Some older directories say Miami; **conflicting**. | [Arcee blog](https://www.arcee.ai/blog/arcee-ai-announces-new-strategic-funding-round) |
| Size | About 26 people in April 2026. Current headcount **not found**. | [TechCrunch](https://techcrunch.com/2026/04/07/i-cant-help-rooting-for-tiny-open-source-ai-model-maker-arcee/) |
| Seed | $5.5M total by Jan 2024; merged with MergeKit (Charles Goddard) the following month | [Arcee blog](https://www.arcee.ai/blog/seedroundandmergekitmerger) |
| Series A | $24M, led by Emergence Capital, 2024 | [Arcee blog](https://www.arcee.ai/blog/small-language-models-rising-as-arcee-ai-lands-24m-series-a) |
| Strategic round | Jul 30 2025, led by Prosperity7 and M12, with Hitachi Ventures, Wipro, JC2, Samsung Next. Amount not stated. | [Arcee blog](https://www.arcee.ai/blog/arcee-ai-announces-new-strategic-funding-round) |
| Series B | Sep 16 2026, led by Vista Equity Partners, Cambium Capital, Emergence Capital; with AI10 Ventures, Hitachi, IAG, M12, P7, Wipro. Valuation "more than $1 billion". Amount undisclosed by Arcee; Fortune's source says at least $150M (**reported, not confirmed**). | [Arcee blog](https://www.arcee.ai/blog/arcee-ai-raises-series-b-to-build-the-future-of-american-open-models), [TNW](https://thenextweb.com/news/arcee-ai-series-b-1bn-valuation-open-weight-trinity) |
| Training cost | Entire 2025 model line-up including Trinity Large built for about $20M | [Arcee blog](https://www.arcee.ai/blog/arcee-ai-raises-series-b-to-build-the-future-of-american-open-models) |

Timeline of launches:

- Dec 1 2025: Trinity Mini and Nano ([blog](https://www.arcee.ai/blog/the-trinity-manifesto)).
- Jan 27 2026: Trinity Large Preview, free on OpenRouter during preview ([blog](https://www.arcee.ai/blog/trinity-large)).
- Apr 1 2026: Trinity-Large-Thinking ([blog](https://www.arcee.ai/blog/trinity-large-thinking)). Preview had served 3.37 trillion tokens on OpenRouter in two months (same post).
- Apr 2026: Trinity Builders Program, free inference grants from under 50M to over 1B tokens ([Arcee on X](https://x.com/arcee_ai/status/2044454391705481298)).
- May 29 2026: licence change to OpenMDW-1.1 ([blog](https://www.arcee.ai/blog/trinity-is-moving-to-openmdw-1-1)).
- Jul 22 2026: Genesis-Science-1 with the DOE ([blog](https://www.arcee.ai/blog/genesis-science-1)).
- Aug 13 2026: Open Models API expanded beyond Trinity; nac open-sourced ([API post](https://www.arcee.ai/blog/open-models-api-beta), [nac post](https://www.arcee.ai/blog/nac)).
- Sep 14 2026: Forge with Bolt. Opted-in Bolt users get 50x usage on open models; sessions feed Arcee's training and evals ([blog](https://www.arcee.ai/blog/arcee-partners-with-bolt-on-forge)).
- Sep 16 2026: Series B.

---

## 4. Customers and use cases

Arcee's site currently shows no customer logos ([arcee.ai](https://www.arcee.ai/)). Named relationships I could verify:

- **US Department of Energy and national labs**: Genesis-Science-1 ([blog](https://www.arcee.ai/blog/genesis-science-1)).
- **Bolt**: Forge research preview ([blog](https://www.arcee.ai/blog/arcee-partners-with-bolt-on-forge)).
- **Cisco (retail)**: at NRF 2026, Cisco's booth ran an in-store conversational assistant "powered by a retail-focused small language model (SLM) from Arcee AI" on Cisco Unified Edge with Intel Xeon 6 ([Cisco blog, Feb 19 2026](https://blogs.cisco.com/retail/when-retail-ai-meets-the-store-floor)). This is the one retail/shopping reference and worth mentioning to the judges.
- **Kilo Code, OpenClaw, Hermes Agent, OpenCode, Cline**: agent tools Trinity is tuned for or promoted in ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md), [Kilo blog](https://blog.kilo.ai/p/trinity-large-thinking-is-free-in)).
- **Partners on training**: DatologyAI (data), Prime Intellect (compute), NVIDIA ([blog](https://www.arcee.ai/blog/the-trinity-manifesto), [NVIDIA case study](https://www.nvidia.com/en-us/case-studies/arcee-ai/)).
- **Strategic investors with go-to-market roles**: Wipro, Hitachi, M12, Samsung, Aramco networks ([blog](https://www.arcee.ai/blog/arcee-ai-announces-new-strategic-funding-round)).

Target verticals in Arcee's words: regulated enterprises in healthcare, financial services, telecom and public sector; industrial and energy operators; software providers and system integrators ([blog](https://www.arcee.ai/blog/arcee-ai-announces-new-strategic-funding-round)). Formal customer case studies with metrics: **not found**.

---

## 5. Why they are at this hackathon

Inferred from recent launches and the prize wording:

- **Drive usage of the Open Models API.** The prize is paid in "platform credits" ([event page](https://tokensand.com/personalagenthack)). The API went multi-model three months ago and Arcee wants developers on it.
- **Learn model choice per task.** Arcee says it wants to know "which models users prefer for a given task, and more importantly, why" to shape future Trinity models ([blog](https://www.arcee.ai/blog/open-models-api-beta)). A project that documents why each step uses each model gives them exactly this.
- **Agents are the target workload.** Trinity-Large-Thinking is "agentic-first", built for "tool calling, multi-step planning, and agent workflows" ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md)). A personal-agents hackathon fits.
- **Cost story.** Their headline claim is frontier-adjacent quality at about 96% lower cost ([blog](https://www.arcee.ai/blog/trinity-large-thinking)). "Pareto frontier" is a natural name for a prize that rewards showing this trade-off.
- **Eval-minded product lead.** Sam Fraser's background is competitive model evaluation and experimentation (section 6), so measured comparisons should land better than a bare integration.

---

## 6. Prize track

**Exact wording** ([event page](https://tokensand.com/personalagenthack)):

- Sponsor line: "Open-weight foundation models."
- Prize: "Most Impressive Use of the Pareto Frontier"
- 1st: $500 platform credits + $250 Arcee Merch Store gift card
- 2nd: $250 platform credits + $100 gift card
- 3rd: $100 gift card
- Link supplied: https://docs.arcee.ai/
- Judging criteria: none published.

**People attending**

- **Sam Fraser, Product (judge and speaker).** Arcee's X account calls her "our head of product" who "works at the seam between our models and our products" and announced she made a 2026 "100 Women in AI" list (as returned by search; I could not open X directly). Her profile describes earlier work as a competitive eval lead at Microsoft AI and experimentation at Bing; LinkedIn headline "Data Science-y PM" ([X profile](https://x.com/samfrannn), [LinkedIn](https://www.linkedin.com/in/samanthaafraser/), [Flybridge list](https://www.flybridge.com/ideas/the-bow/announcing-the-2026-100-women-in-ai)). Which "100 Women in AI" list: **unverified**. Takeaway: she is likely to care about evals, experiments and honest numbers.
- **Anneketh Vij, AI Research Scientist (judge).** GitHub bio: "AI Research Scientist | MS CS AI @ USC", California, member of the arcee-ai org ([GitHub](https://github.com/annekethvij)). Hugging Face profile lists Arcee AI and two papers: the Arcee Trinity Large Technical Report (arXiv 2602.17004) and a 2024 paper on domain adaptation of Llama3-70B by continual pre-training (arXiv 2406.14971); recent activity is updating the Trinity model cards ([HF profile](https://huggingface.co/annekethvij), [HF papers](https://huggingface.co/annekethvij/papers)). A third-party index credits the "Introducing the Trinity Builders Program" blog post to this name ([plushcap](https://www.plushcap.com/companies/arcee-ai/blog/2025)); **unverified** on Arcee's own site. Takeaway: knows the models' internals, tool-calling behaviour and reasoning-trace handling; will notice if `reasoning_content` is dropped.

---

## 7. What they expect to see used (ranked by likely impact)

1. **Several Arcee-hosted models used deliberately, with a measured cost/quality trade-off.** This is the literal reading of the prize. Route each step to the cheapest adequate model and show the numbers.
2. **Trinity-Large-Thinking as the agent's reasoning core** with correct `reasoning_content` round-tripping. It is their flagship and "agentic-first" ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md)). Confirm it is live first (section 11).
3. **Tool calling and JSON output through the Arcee API** (`tools`, `tool_choice`, `response_format`) ([function calling](https://docs.arcee.ai/capabilities/function-calling.md), [structured outputs](https://docs.arcee.ai/capabilities/structured-outputs.md)).
4. **Reasoning traces surfaced in the audit trail.** The API exposes the model's thinking as a separate field ([reasoning traces](https://docs.arcee.ai/capabilities/reasoning-traces.md)); showing it per decision ties the Arcee track to the main challenge.
5. **`GET /api/v1/models` and `GET /api/v1/usage/stats`** to pull live prices and real usage into the cost report ([models](https://docs.arcee.ai/api-reference/models.md), [usage](https://docs.arcee.ai/api-reference/usage.md)).
6. **Open weights run locally** (Trinity Nano or Mini via Ollama, llama.cpp or vLLM) as the zero-marginal-cost point on the frontier ([docs](https://docs.arcee.ai/quick-deploys/ollama.md)). Impressive but risky in 5.5 hours.
7. **nac** as the harness ([GitHub](https://github.com/arcee-ai/nac)). It is built for hours-long engineering tasks, so it is a poor fit for a shopping agent; mention only.

---

## 8. Quickstart

### Signup and auth

1. Register at [platform.arcee.ai](https://platform.arcee.ai/).
2. Top up in [Wallet](https://platform.arcee.ai/api/wallet) "if needed".
3. Create a key at [API Keys](https://platform.arcee.ai/api/api-keys). Keys are prefixed `rcai-` ([chat completion spec](https://docs.arcee.ai/api-reference/chat-completion.md)).
4. Base URL: `https://api.arcee.ai/api/v1`, bearer auth, OpenAI-compatible ([quick start](https://docs.arcee.ai/get-started/quick-start.md)).

### Models and prices on the Arcee API

The official pricing page, fetched 2026-11-19 ([docs pricing](https://docs.arcee.ai/get-started/pricing.md)), USD per 1M tokens:

| Model ID | Input | Output | Cached input |
|---|---|---|---|
| `zai-org/glm-5.3-flash` | $0.15 | $0.50 | $0.03 |
| `deepseek/deepseek-v4.1-flash` | $0.22 | $1.20 | $0.007 |
| `deepseek/deepseek-v4.1-flash-1017` | $0.30 | $1.20 | $0.006 |
| `zai-org/glm-5.3` | $1.40 | $4.40 | $0.26 |
| `moonshotai/kimi-k3.1-pro` | $2.00 | $10.00 | $0.20 |
| `moonshotai/kimi-k3` | $3.00 | $15.00 | $0.30 |

Additional entries for the Arcee provider in the models.dev catalog, which nac uses ([models.dev](https://models.dev/api.json), [nac catalog docs](https://github.com/arcee-ai/nac/blob/main/docs/configuration/catalog.md)). Third-party data, may be stale:

| Model ID | Input | Output | Context | Tools | Notes |
|---|---|---|---|---|---|
| `trinity-large-thinking` | $0.25 | $0.80 | 262,144 | yes | Reasoning model. Same price in Arcee's [Aug 13 post](https://www.arcee.ai/blog/open-models-api-beta). |
| `deepseek/deepseek-v4-flash-latest` | $0.14 | $0.28 | 1,048,576 | yes | |
| `thinkingmachines/inkling-small` | $0.50 | $1.20 | 262,144 | yes | |
| `deepseek/deepseek-v4-pro` | $1.74 | $3.48 | 512,000 | yes | |
| `zai-org/glm-5.2` | $1.40 | $4.40 | 262,144 | yes | |

models.dev also lists context windows of about 1M tokens for GLM-5.3, GLM-5.3-Flash, Kimi K3 and DeepSeek V4.1 Flash, and marks every Arcee-provider model as supporting tool calls and reasoning ([models.dev](https://models.dev/api.json)). **Do not hard-code any of this**: call `GET /api/v1/models`, which returns `id`, `context_length`, `max_output_length`, `pricing` and `supported_features` per model ([models API](https://docs.arcee.ai/api-reference/models.md)).

**Conflict to resolve at the booth.** The docs' Trinity Family table lists "API Model Name: Not Hosted" for Nano, Mini and Large-Thinking ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md)), and Trinity is absent from the pricing page. But [arcee.ai/trinity](https://www.arcee.ai/trinity) says Large Thinking deploys on "Arcee API — served at 256K context", and the nac repo (pushed Nov 17 2026) uses `model = "trinity-large-thinking"` throughout ([nac credentials docs](https://github.com/arcee-ai/nac/blob/main/docs/configuration/credentials.md)). Whether any Trinity model is callable on the API today is **unverified** without a key.

### Trinity context windows (model capability, not necessarily hosted)

| Model | Total / active params | Context | Source |
|---|---|---|---|
| Trinity Nano | 6B / 1B | 128k | [docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md) |
| Trinity Mini | 26B / 3B | 128k | [docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md) |
| Trinity Large Thinking | 398B / 13B | 512k (docs say "hosted at 128k"; website says 256k) | [docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md), [website](https://www.arcee.ai/trinity) |

### Other routes to Arcee models

- **OpenRouter.** Arcee's docs show `base_url="https://openrouter.ai/api/v1"` with `model="arcee-ai/trinity-mini"` ([docs](https://docs.arcee.ai/integration-list/openrouter.md)). **But on 2026-11-19 OpenRouter's public API returned zero Arcee models in `/api/v1/models`, and `endpoints: []` for `arcee-ai/trinity-mini`, `trinity-large-thinking`, `trinity-large-preview` and `afm-4.5b`** ([example](https://openrouter.ai/api/v1/models/arcee-ai/trinity-mini/endpoints)). The listing page still shows 10 slugs ([openrouter.ai/arcee-ai](https://openrouter.ai/arcee-ai)). Assume OpenRouter does not serve Arcee models right now; test before relying on it.
- **Other hosts in models.dev** (unverified, third-party): DigitalOcean `arcee-trinity-large-thinking` ($0.25/$0.90, 128k) and Clarifai `arcee_ai/AFM/models/trinity-mini` ($0.045/$0.15, 131k) ([models.dev](https://models.dev/api.json)).
- **Local.** Weights at [huggingface.co/arcee-ai](https://huggingface.co/arcee-ai), including GGUF builds of Mini and Nano; docs cover Ollama, llama.cpp, vLLM and SGLang ([docs index](https://docs.arcee.ai/llms.txt)).
- A prize paid in Arcee platform credits strongly suggests the judges want the **Arcee API**, not a third-party host.

### Minimal call (from the official quick start; only the model ID is filled in)

```python
from openai import OpenAI

client = OpenAI(
    api_key="YOUR_API_KEY",
    base_url="https://api.arcee.ai/api/v1"
)

response = client.chat.completions.create(
    model="zai-org/glm-5.3-flash",   # docs use the placeholder "MODEL_NAME"
    messages=[
        {"role": "system", "content": "You are a helpful assistant"},
        {"role": "user", "content": "What is 25 + 37?"},
    ],
    stream=False
)

print(response.choices[0].message.content)
print(response.choices[0].message.reasoning_content)
```

Source: [quick start](https://docs.arcee.ai/get-started/quick-start.md).

### Tool calling (adapted from the docs' agent loop; tool names changed to a shopping example)

The loop structure, `build_assistant_message` and the `reasoning_content` handling are copied from [Reasoning Traces](https://docs.arcee.ai/capabilities/reasoning-traces.md). The docs example points at a local vLLM server; the base URL here is the hosted one from the quick start.

```python
import json
from openai import OpenAI

client = OpenAI(base_url="https://api.arcee.ai/api/v1", api_key="YOUR_API_KEY")
MODEL = "trinity-large-thinking"   # confirm via GET /api/v1/models

tools = [
    {
        "type": "function",
        "function": {
            "name": "search_products",
            "description": "Search the catalog for products matching a query.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search text"}
                },
                "required": ["query"]
            }
        }
    }
]

def build_assistant_message(msg) -> dict:
    assistant_msg = {
        "role": "assistant",
        "content": msg.content or "",  # never null
    }
    # Preserve reasoning — critical for multi-turn
    reasoning = getattr(msg, "reasoning_content", None) or getattr(msg, "reasoning", None)
    if reasoning:
        assistant_msg["reasoning_content"] = reasoning
    # Preserve tool calls
    if msg.tool_calls:
        assistant_msg["tool_calls"] = [
            {
                "id": tc.id,
                "type": "function",
                "function": {"name": tc.function.name, "arguments": tc.function.arguments}
            }
            for tc in msg.tool_calls
        ]
    return assistant_msg

def run_agent(user_message: str, system_prompt: str):
    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": user_message}
    ]
    for step in range(10):  # safety limit
        response = client.chat.completions.create(
            model=MODEL, messages=messages, tools=tools,
            tool_choice="auto", temperature=0, max_tokens=1000
        )
        msg = response.choices[0].message
        messages.append(build_assistant_message(msg))
        if not msg.tool_calls:
            return msg.content or ""
        for tc in msg.tool_calls:
            result = execute_tool(tc.function.name, tc.function.arguments)  # your code
            messages.append({"role": "tool", "tool_call_id": tc.id, "content": result})
    raise RuntimeError("Agent exceeded maximum steps")
```

API facts from the spec ([chat completion](https://docs.arcee.ai/api-reference/chat-completion.md)): `tool_choice` accepts `auto`, `none`, `required`; only function tools are supported; the response carries standard `usage`; streaming is OpenAI-style SSE; error codes include 402 (insufficient balance), 429 (rate limit) and 503 (overloaded).

### Structured output (from the docs; model ID filled in)

```python
import json
from openai import OpenAI

client = OpenAI(api_key="<YOUR_API_KEY>", base_url="https://api.arcee.ai/api/v1")

system_prompt = """
The user will provide some exam text. Please parse the "question" and "answer" and output them in JSON format.

EXAMPLE INPUT:
Which is the highest mountain in the world? Mount Everest.

EXAMPLE JSON OUTPUT:
{
    "question": "What is the capital of France?",
    "answer": "Paris"
}
"""

response = client.chat.completions.create(
    model="zai-org/glm-5.3-flash",   # docs use "MODEL_NAME"
    messages=[{"role": "system", "content": system_prompt},
              {"role": "user", "content": "What is the capital of Italy?"}],
    response_format={'type': 'json_object'}
)
print(json.loads(response.choices[0].message.content))
```

Source: [structured outputs](https://docs.arcee.ai/capabilities/structured-outputs.md). The docs require the word "json" in the prompt, an example of the target shape, and a sensible `max_tokens` so the JSON is not cut off. The page mentions schema-validated output with Pydantic but shows no code for it, and `response_format` is not in the published request schema, so `json_schema` mode is **unverified**; validate with Pydantic on your side.

### Free tier, credits, rate limits, gotchas

- **Free credits for attendees: not found.** The event page lists credits only as prizes. Ask the booth for a hackathon code. The quick start says to top up the wallet "if needed" ([docs](https://docs.arcee.ai/get-started/quick-start.md)).
- The $20 signup credit in the docs is for Conductor, which looks retired ([docs](https://docs.arcee.ai/arcee-conductor/introduction-to-arcee-conductor.md)).
- Rate limits: not published. The API returns 429, and the platform app has per-key rate-limit profiles (requests and tokens per minute/day) ([spec](https://docs.arcee.ai/api-reference/chat-completion.md)).
- **Reasoning round-trip.** Dropping `reasoning_content` between turns "may produce malformed output". Send `content: ""`, never `null` ([docs](https://docs.arcee.ai/capabilities/reasoning-traces.md)).
- Recommended sampling: Trinity-Large-Thinking temperature 0.3; Mini/Nano temperature 0.15, top_p 0.75, top_k 50, min_p 0.06 ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md), [docs](https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-mini-26b.md)).
- Trinity knowledge cutoff is 2024 ([docs](https://docs.arcee.ai/arcee-language-models/trinity-family.md)); give it product facts via tools or context.
- Cloud model versions can be retired with as little as 5 business days' notice ([deprecation policy](https://docs.arcee.ai/policies/deprecation-policy.md)); the docs and the pricing page already disagree, so check the live model list on the day.
- The docs' function-calling example passes `http_client=httpx.Client(http2=True)` for speed ([docs](https://docs.arcee.ai/capabilities/function-calling.md)); optional.
- Path note: the OpenAPI spec shows server `https://api.arcee.ai` with path `/v1/chat/completions`, while the quick start uses `https://api.arcee.ai/api/v1/chat/completions`. Use the quick start URL.

---

## 9. Past hackathons this company sponsored

- **Arcee/MergeKit Model Merging Hackathon** (online), Apr 19 to May 13 2024, co-sponsored by AWS, about $9,000 in cash prizes. Categories included best new merge and best integration with other ecosystems; judges Charles Goddard and Mark McQuade ([Arcee blog](https://www.arcee.ai/blog/arcee-mergekit-launch-model-merging-hackathon), [PR Newswire](https://www.prnewswire.com//news-releases/arceeai-advances-model-merging-innovations-with-launch-of-mergekit-hackathon-302121568.html)). **Winning projects: not found.**
- **Any 2025 or 2026 hackathon with an Arcee prize track, and any winners: not found.** Searches for Arcee, Trinity and Conductor prize tracks on Devpost, lu.ma and lablab returned nothing.
- Related, not a hackathon: the Trinity Builders Program gives token grants to developers and open-source builders ([Arcee on X](https://x.com/arcee_ai/status/2044454391705481298)).

**Patterns in what wins:** no winner data exists to draw from. The only evidence is what Arcee itself celebrates: agent benchmarks (PinchBench, Tau2), cost per token against closed models, and real usage in agent harnesses ([blog](https://www.arcee.ai/blog/trinity-large-thinking)). A project that reports a task-success number and a dollar figure side by side matches that style.

---

## 10. Fit with "shopping agent with an audit trail"

The audit trail and the Pareto story are the same data: for every step, record which model ran, why it was picked, tokens, cost, latency, and the reasoning trace.

| # | Idea | What it shows | Time in a 5.5h build |
|---|---|---|---|
| 1 | **Single-model swap.** Run the agent's tool loop on one Arcee-hosted model. Log `usage` and computed cost per call. | Qualifies. Not a frontier story on its own. | 30 min |
| 2 | **Two-tier routing by step type.** A cheap fast model (for example `zai-org/glm-5.3-flash` at $0.15/$0.50) handles intent parsing, attribute extraction and policy lookups. A stronger model (Trinity-Large-Thinking if live, otherwise `zai-org/glm-5.3`) handles the compare-and-decide step and the purchase confirmation. Audit log shows model, tokens, dollars, latency per step. | Deliberate model choice with numbers. | 1 to 1.5 h |
| 3 | **Frontier plot from a mini-eval.** Write 10 to 15 fixed shopping tasks with known correct outcomes (right item, within budget, policy respected). Run each candidate model and the routed policy; plot cost per task against success rate, draw the frontier, show the routed policy on it and "everything on Kimi K3" as dominated. Put the chart in the demo video. | The literal prize: evidence of being on the Pareto frontier. Appeals to an eval-background judge. | +1.5 h on top of idea 2 (2.5 to 3 h total); the eval can run in the background while the UI is built |
| 4 | **Escalation with a verifier.** Cheap model answers first; a deterministic check (schema valid, price within budget, claim backed by a retrieved context ID) decides whether to escalate to the stronger model. Log every escalation and its reason. Report "X% of steps stayed on the cheap tier, cost per purchase $Y vs $Z". | Dynamic routing, cost-aware and auditable. Strongest tie to "reliable context" in the challenge. | +1 h on top of idea 2 |
| 5 | **Reasoning in the audit trail.** Store `reasoning_content` for each decision next to the retrieved context IDs and show it in an expandable "why" panel. | Uses a Trinity-specific feature and directly serves challenge point 3. | 30 to 45 min, only if a reasoning model is live |

Recommended plan: ideas 2 + 3 + 5, with 4 if time allows. Pull prices from `GET /api/v1/models` rather than a hard-coded table so the cost figures are defensible. Use real measured latency and token counts; do not estimate.

Deep option to skip: running Trinity Nano locally as a zero-cost tier. It makes a striking frontier point, but setup and tool-calling reliability on a laptop are a real risk in 5.5 hours.

---

## 11. Risks and open questions for the booth

1. **What exactly do you mean by "the Pareto frontier"?** Cost vs quality across models on your API? Trinity vs closed models? Something else? The phrase is not in any Arcee material I could find.
2. **Which models are live on `api.arcee.ai` today?** Is `trinity-large-thinking` callable? The docs say "Not Hosted", the website and nac say it is. Are Trinity Mini or Nano hosted anywhere official?
3. **Does the project need to use a Trinity model to qualify**, or does any model on the Arcee API count?
4. **Are there free credits or a promo code for attendees?** Otherwise a card top-up is needed before the first call.
5. **Rate limits** for new accounts, and can they be raised for the day? An eval sweep makes many calls.
6. **Is `response_format` with a JSON schema supported**, or only `json_object`? Does it vary by model?
7. **Do the third-party models return `reasoning_content` the same way**, and must it be passed back for them too?
8. **Is OpenRouter still a supported route?** It showed no live Arcee endpoints on Nov 19.
9. **Is Conductor still available?** If so, would using its `auto` routing count for or against a custom router?
10. **How will the track be judged?** Demo only, or will they look at the repo and the numbers?

Risks:

- The prize meaning is inferred. Mitigation: ask at 9:30 AM; the per-step cost log is useful for the main challenge either way.
- Model availability can change at short notice ([deprecation policy](https://docs.arcee.ai/policies/deprecation-policy.md)). Mitigation: keep model IDs in config and read the live list.
- Data routing: some hosted models are from Chinese labs; Arcee's privacy terms for third-party models were not reviewed. Use fictional shopper data.
- Prize value is modest (top prize $500 credits + $250 merch), so do not let this track drive the architecture; ideas 2 and 3 cost about 3 hours and also strengthen the audit trail for the main judges.

---

## 12. Sources

Arcee docs
- https://docs.arcee.ai/
- https://docs.arcee.ai/llms.txt
- https://docs.arcee.ai/llms-full.txt
- https://docs.arcee.ai/get-started/quick-start.md
- https://docs.arcee.ai/get-started/pricing.md
- https://docs.arcee.ai/arcee-language-models/trinity-family.md
- https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-nano-6b.md
- https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-mini-26b.md
- https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-preview.md
- https://docs.arcee.ai/arcee-language-models/trinity-family/trinity-large-thinking.md
- https://docs.arcee.ai/api-reference/chat-completion.md
- https://docs.arcee.ai/api-reference/models.md
- https://docs.arcee.ai/api-reference/usage.md
- https://docs.arcee.ai/capabilities/function-calling.md
- https://docs.arcee.ai/capabilities/structured-outputs.md
- https://docs.arcee.ai/capabilities/reasoning-traces.md
- https://docs.arcee.ai/integration-list/openrouter.md
- https://docs.arcee.ai/arcee-conductor/introduction-to-arcee-conductor.md
- https://docs.arcee.ai/arcee-conductor/features-and-functionality/api.md
- https://docs.arcee.ai/arcee-conductor/pricing.md
- https://docs.arcee.ai/other/post-trains.md
- https://docs.arcee.ai/policies/deprecation-policy.md
- https://docs.arcee.ai/quick-deploys/ollama.md

Arcee site and blog
- https://www.arcee.ai/
- https://www.arcee.ai/trinity
- https://www.arcee.ai/press
- https://www.arcee.ai/blog
- https://www.arcee.ai/blog/open-models-api-beta
- https://www.arcee.ai/blog/nac
- https://www.arcee.ai/blog/arcee-ai-raises-series-b-to-build-the-future-of-american-open-models
- https://www.arcee.ai/blog/arcee-partners-with-bolt-on-forge
- https://www.arcee.ai/blog/genesis-science-1
- https://www.arcee.ai/blog/trinity-is-moving-to-openmdw-1-1
- https://www.arcee.ai/blog/trinity-large-thinking
- https://www.arcee.ai/blog/trinity-large
- https://www.arcee.ai/blog/the-trinity-manifesto
- https://www.arcee.ai/blog/arcee-ai-announces-new-strategic-funding-round
- https://www.arcee.ai/blog/seedroundandmergekitmerger
- https://www.arcee.ai/blog/small-language-models-rising-as-arcee-ai-lands-24m-series-a
- https://www.arcee.ai/blog/arcee-mergekit-launch-model-merging-hackathon
- https://platform.arcee.ai/

Code, models, catalogs
- https://github.com/arcee-ai
- https://github.com/arcee-ai/nac
- https://github.com/arcee-ai/nac/blob/main/docs/configuration/credentials.md
- https://github.com/arcee-ai/nac/blob/main/docs/configuration/catalog.md
- https://huggingface.co/arcee-ai
- https://huggingface.co/arcee-ai/Trinity-Large-Thinking
- https://huggingface.co/arcee-ai/Trinity-Mini
- https://models.dev/api.json
- https://openrouter.ai/arcee-ai
- https://openrouter.ai/api/v1/models
- https://openrouter.ai/api/v1/models/arcee-ai/trinity-mini/endpoints

People
- https://github.com/annekethvij
- https://huggingface.co/annekethvij
- https://huggingface.co/annekethvij/papers
- https://www.plushcap.com/companies/arcee-ai/blog/2025
- https://x.com/samfrannn (via search snippet; direct fetch blocked)
- https://www.linkedin.com/in/samanthaafraser/ (via search snippet)
- https://www.flybridge.com/ideas/the-bow/announcing-the-2026-100-women-in-ai
- https://x.com/arcee_ai/status/2044454391705481298 (via search snippet)

Press and third parties
- https://tokensand.com/personalagenthack
- https://thenextweb.com/news/arcee-ai-series-b-1bn-valuation-open-weight-trinity
- https://siliconangle.com/2026/09/16/open-weight-model-developer-arcee-ai-reaches-1b-plus-valuation-with-undisclosed-series-b-funding/
- https://techcrunch.com/2026/04/07/i-cant-help-rooting-for-tiny-open-source-ai-model-maker-arcee/
- https://blogs.cisco.com/retail/when-retail-ai-meets-the-store-floor
- https://www.nvidia.com/en-us/case-studies/arcee-ai/
- https://blog.kilo.ai/p/trinity-large-thinking-is-free-in
- https://www.prnewswire.com//news-releases/arceeai-advances-model-merging-innovations-with-launch-of-mergekit-hackathon-302121568.html
- https://arxiv.org/pdf/2511.23455
