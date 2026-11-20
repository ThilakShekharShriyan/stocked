# Reference projects — what to borrow

18 past hackathon projects (Cerebral Valley events, 2026), read from their repos and gallery pages on Nov 20. Awards come from gallery pages and team posts only; no independent winners lists were found. No demo videos were watched and no code was run.

| File | Projects |
|---|---|
| [group-a-closest-matches.md](group-a-closest-matches.md) | supplier-chief-of-staff, Mato, eagle., Rosetta, aie-hack (= Lanes) |
| [group-b-meta-hackathon.md](group-b-meta-hackathon.md) | Punchwalk, Formwright, Voluma Bio, oqed, synthexis |
| [group-c-verification-patterns.md](group-c-verification-patterns.md) | Etch, AGI Factory, SPARKLETEAM, Frankenstein, LabTwin, Morf, Labbit |

**Code not readable:** Mato (404), eagle. (private), LabTwin (404), Morf (404) — gallery text only.

## The shared pattern

The model proposes, code verifies, and the result is one measured before/after number — often alongside a failure the system caught.

## Patterns mapped to the household agent

| # | Pattern | From | Household-agent use | Sponsor |
|---|---|---|---|---|
| 1 | Trigger in code: scheduled check of days-of-supply vs reorder point; model called only when something runs low | supplier-chief-of-staff | SQL job over inventory | SingleStore |
| 2 | "Done" checks written before acting; numbers recomputed from the DB, never from model text | supplier-chief-of-staff, Etch, Formwright | Budget, quantity, approver, delivery date checked in code before ordering | SingleStore |
| 3 | Done gate: when the model says finish, code returns reasons and the loop continues; after 3 blocks ship with open problems listed | Formwright | Plan → checks → re-plan | Firedrill |
| 4 | "Cannot measure" instead of guessing | Etch | "I don't know how much rice is left" | — |
| 5 | Send-or-hold: items passing a separate check go out; the rest are held with a one-line reason | Punchwalk | Under spend limit auto-orders; over limit waits for approval | Statsig (limit as config) |
| 6 | One-text approval answered with "Y" | eagle., supplier-chief-of-staff (Telegram) | Approve/Skip message to the right housemate | — (build it; none of the readable repos do reply-to-approve) |
| 7 | Chat messages become stored rules linked to the source message | Mato | House preferences with a citation | SingleStore or Senso |
| 8 | Provenance per output; code lists what a change would make stale | Voluma | "Why" panel; a new party date invalidates the old plan | SingleStore |
| 9 | Append-only event table feeding ledger, analytics and a cached demo replay | SPARKLETEAM, Punchwalk | Single source for the audit trail | SingleStore → Amplitude |
| 10 | Label each ledger row observed vs inferred; confidence per learned belief | SPARKLETEAM | "Sam usually buys oat milk (4 of 5 times)" | — |
| 11 | Typed rejection stages as a funnel | oqed | Where plans fail: no stock, over budget, rejected | Amplitude |
| 12 | Keep a change only if it wins on unseen runs with zero new unsafe actions | AGI Factory (66.5→92.8, unsafe 21→0) | Agent v1 vs v2 on the same simulated housemates | Firedrill + Statsig |
| 13 | Show a rejected shortcut or a caught false win | AGI Factory, SPARKLETEAM, Frankenstein | The order the agent refused to place | Demo beat |
| 14 | Order path sends only the checked version, with an idempotency key | Formwright | No double charge on retry | Firedrill exact check |
| 15 | Big model labels offline, small models run live | synthexis, Lanes/aie-hack (18% cost saving) | Classify receipts/items with a small model | Superlinked SIE |
