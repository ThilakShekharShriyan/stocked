"use client";

// Supplies pipeline, audit trail and the scenario lab.

import { useEffect, useMemo, useState } from "react";
import { AgentEvent, House, api, dayLabel, latency, money, nameOf, timeLabel } from "@/lib/api";
import { WorldNode, autonomy, senderOf, simulationLabel, toneOf, TONE_HEX } from "@/lib/world";
import { Badge, Card } from "./overview";
import { SessionCard, Session } from "./tabs";

// ---------------------------------------------------------------------------------------------
// Supplies: the path a purchase takes, each stage showing the real records behind it.

const STAGES = [
  { id: "inbox", label: "Receipt inbox", node: "inbox" },
  { id: "reader", label: "Receipt reader", node: "reader" },
  { id: "classifier", label: "Item classifier", node: "classifier" },
  { id: "estimator", label: "Run-out estimator", node: "estimator" },
  { id: "planner", label: "Basket planner", node: "planner" },
  { id: "rules", label: "Budget and rules", node: "rules" },
  { id: "decision", label: "Decision", node: "hub" },
  { id: "order", label: "Test-mode order", node: "payments" },
  { id: "split", label: "Cost split", node: "hub" },
  { id: "audit", label: "Audit event", node: "audit" },
] as const;
type StageId = (typeof STAGES)[number]["id"];

function Rows({ rows, empty }: { rows: [string, React.ReactNode, string?][]; empty: string }) {
  if (rows.length === 0) return <p className="text-sm text-stone-500">{empty}</p>;
  return (
    <ul className="divide-y divide-line">
      {rows.map(([a, b, c], i) => (
        <li key={i} className="flex items-baseline justify-between gap-4 py-1.5 text-sm">
          <span className="min-w-0">
            <span className="font-medium">{a}</span>
            {c && <span className="block truncate text-xs text-stone-500">{c}</span>}
          </span>
          <span className="shrink-0 text-right text-stone-600">{b}</span>
        </li>
      ))}
    </ul>
  );
}

export function SuppliesFlow({ house, onWhy, onStage }: { house: House; onWhy: (id: string) => void; onStage: (node: string) => void }) {
  const [stage, setStage] = useState<StageId>("estimator");
  const ev = house.recent_events;
  const of = (kind: string) => ev.filter((e) => e.kind === kind);
  const a = autonomy(house);
  const supplyPlans = house.plans;
  const latest = supplyPlans.find((p) => p.basket.length > 0);
  const tracked = house.items.filter((i) => i.runs_out_on);
  const count: Record<StageId, number> = {
    inbox: of("receipt_ingested").length,
    reader: of("line_extracted").length,
    classifier: of("decision_gate").length,
    estimator: tracked.length,
    planner: supplyPlans.filter((p) => p.basket.length > 0).length,
    rules: supplyPlans.reduce((n, p) => n + p.verdict.checks.length, 0),
    decision: supplyPlans.length,
    order: a.testOrders,
    split: house.settlements.length,
    audit: ev.length,
  };

  let body: React.ReactNode;
  switch (stage) {
    case "inbox":
      body = <Rows empty="No receipts synced since the last reset. Press “Sync inbox”." rows={of("receipt_ingested").map((e) => [e.summary, String(e.data.source ?? ""), `message ${String(e.data.source_ref ?? "—")} · ${timeLabel(e.ts)}`])} />;
      break;
    case "reader":
      body = <Rows empty="No lines read from new receipts yet." rows={of("line_extracted").map((e) => [e.summary, e.data.verified ? <Badge tone="sage">verified</Badge> : <Badge tone="clay">excluded</Badge>, `“${String(e.data.span ?? "")}” · ${e.model ?? String(e.data.extractor ?? "")}${e.latency_us ? ` · ${latency(e.latency_us)}` : ""}`])} />;
      break;
    case "classifier":
      body = <Rows empty="No item has been classified yet. A flatmate asking the house for something triggers it." rows={of("decision_gate").map((e) => [e.summary, e.data.probability !== undefined ? `${Math.round(Number(e.data.probability) * 100)}%` : "", `${e.model ?? ""}${e.latency_us ? ` · ${latency(e.latency_us)}` : ""}`])} />;
      break;
    case "estimator":
      body = <Rows empty="No purchase history yet." rows={tracked.map((i) => [i.name, <button key={i.id} onClick={() => onWhy(i.id)} className="underline decoration-stone-400 underline-offset-2">{i.days_left !== null && i.days_left < 0.5 ? "now" : dayLabel(i.runs_out_on)}</button>, `${i.basis} · ${i.provenance === "observed" ? "observed by a flatmate" : "inferred from receipts, in SQL"}`])} />;
      break;
    case "planner":
      body = (
        <>
          <p className="mb-2 text-sm text-stone-600">Planner variant: <b className="text-ivory">{house.budget.agent_version}</b> ({house.budget.config_source === "statsig" ? "set by a Statsig gate" : "default"}).</p>
          <Rows empty="No baskets proposed yet." rows={supplyPlans.filter((p) => p.basket.length > 0).map((p) => [p.title, <button key={p.id} onClick={() => onWhy(p.id)} className="underline decoration-stone-400 underline-offset-2">{money(p.total_cents)}</button>, p.basket.map((l) => `${l.name} ×${l.qty}`).join(", ")])} />
        </>
      );
      break;
    case "rules":
      body = latest ? (
        <>
          <p className="mb-2 text-sm text-stone-600">Checks run in code on “{latest.title}”. Auto-approve limit {money(house.budget.auto_limit_cents)}; {money(house.budget.remaining_cents)} of {money(house.budget.monthly_cents)} left this month.</p>
          <ul className="space-y-1.5">
            {latest.verdict.checks.map((c) => (
              <li key={c.name} className="flex gap-2 text-sm">
                <span className={`w-4 shrink-0 font-bold ${c.status === "pass" ? "text-emerald-700" : c.status === "fail" ? "text-red-700" : "text-amber-700"}`} aria-label={c.status}>{c.status === "pass" ? "✓" : c.status === "fail" ? "✕" : "?"}</span>
                <span>{c.detail}{c.status === "cannot_measure" && <span className="ml-1 font-semibold text-amber-700">(cannot measure)</span>}</span>
              </li>
            ))}
          </ul>
        </>
      ) : <p className="text-sm text-stone-500">No plan has been checked yet.</p>;
      break;
    case "decision":
      body = <Rows empty="No decisions yet." rows={supplyPlans.map((p) => [p.title, <Badge key={p.id} tone={p.status === "ordered" ? "sage" : p.status === "held" ? "ember" : p.status === "refused" ? "clay" : "ivory"}>{p.status.replaceAll("_", " ")}</Badge>, p.decision_note ?? ""])} />;
      break;
    case "order":
      body = <Rows empty="No test-mode orders yet." rows={house.orders.filter((o) => !o.plan_id.startsWith("out_")).map((o) => [house.plans.find((p) => p.id === o.plan_id)?.title ?? o.plan_id, <span key={o.id}>{money(o.total_cents)} <Badge tone="violet">TEST MODE</Badge></span>, `${o.payment_provider.replaceAll("_", " ")} · ${o.payment_ref}`])} />;
      break;
    case "split":
      body = <Rows empty="Everyone is square." rows={house.settlements.map((s) => [`${nameOf(house.members, s.from)} owes ${nameOf(house.members, s.to)}`, money(s.cents)])} />;
      break;
    default:
      body = <Rows empty="No events yet." rows={ev.slice(0, 8).map((e) => [e.summary, timeLabel(e.ts), e.kind.replaceAll("_", " ")])} />;
  }

  return (
    <Card title="How a purchase happens" hint="select a stage">
      <ol className="flex flex-wrap items-stretch gap-1.5" aria-label="Supplies pipeline">
        {STAGES.map((s, i) => (
          <li key={s.id} className="flex items-center gap-1.5">
            <button
              onClick={() => { setStage(s.id); onStage(s.node); }}
              aria-pressed={stage === s.id}
              className={`rounded-xl border px-3 py-2 text-left ${stage === s.id ? "border-ember bg-raised" : "border-line hover:bg-raised"}`}
            >
              <span className="block text-[10px] uppercase tracking-widest text-stone-500">Step {i + 1}</span>
              <span className="block text-sm font-medium">{s.label}</span>
              <span className="block text-xs tabular-nums text-stone-500">{count[s.id]} recorded</span>
            </button>
            {i < STAGES.length - 1 && <span className="text-stone-400" aria-hidden="true">→</span>}
          </li>
        ))}
      </ol>
      <div className="mt-4 rounded-xl border border-line bg-ink/40 p-4">
        <h3 className="mb-2 text-sm font-semibold">{STAGES.find((s) => s.id === stage)?.label}</h3>
        {body}
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------------------------
// Audit trail

function workflowOf(e: AgentEvent): string {
  if (e.subject_type === "outing") return "Outings";
  if (e.subject_type === "session") return "Conversation";
  if (/verif|identity|blocked_unverified/.test(e.kind)) return "Identity";
  return "Supplies";
}

export function AuditTrail({ house, nodes, onWhy }: { house: House; nodes: WorldNode[]; onWhy: (id: string) => void }) {
  const [flow, setFlow] = useState("All");
  const [q, setQ] = useState("");
  const rows = useMemo(
    () => house.recent_events.filter((e) => (flow === "All" || workflowOf(e) === flow) && (!q || `${e.summary} ${e.kind} ${e.actor}`.toLowerCase().includes(q.toLowerCase()))),
    [house.recent_events, flow, q],
  );
  return (
    <Card title="Audit trail" hint={`${rows.length} of ${house.recent_events.length} most recent events`}>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <div className="flex gap-1 rounded-lg border border-line p-1" role="group" aria-label="Filter by workflow">
          {["All", "Supplies", "Outings", "Identity", "Conversation"].map((f) => (
            <button key={f} onClick={() => setFlow(f)} aria-pressed={flow === f} className={`rounded-md px-2.5 py-1 text-xs font-medium ${flow === f ? "bg-raised text-ivory" : "text-stone-500 hover:text-ivory"}`}>{f}</button>
          ))}
        </div>
        <label className="sr-only" htmlFor="audit-q">Search the audit trail</label>
        <input id="audit-q" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search events…" className="min-h-9 flex-1 rounded-lg border border-line bg-ink px-3 text-sm placeholder:text-stone-400" />
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[820px] text-left text-sm">
          <thead className="text-[11px] uppercase tracking-widest text-stone-500">
            <tr>
              <th className="py-2 pr-3 font-semibold">Time</th>
              <th className="py-2 pr-3 font-semibold">Who</th>
              <th className="py-2 pr-3 font-semibold">From → to</th>
              <th className="py-2 pr-3 font-semibold">What happened</th>
              <th className="py-2 pr-3 font-semibold">Workflow</th>
              <th className="py-2 pr-3 font-semibold">Model · time</th>
              <th className="py-2 font-semibold">Real or simulated</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((e) => {
              const s = senderOf(e, house, nodes);
              const sim = simulationLabel(e);
              return (
                <tr key={e.id} className="border-t border-line align-top hover:bg-raised">
                  <td className="whitespace-nowrap py-2 pr-3 tabular-nums text-stone-500">{timeLabel(e.ts)}</td>
                  <td className="py-2 pr-3">{e.actor === "agent" ? "Stocked" : nameOf(house.members, e.actor) || e.actor}</td>
                  <td className="whitespace-nowrap py-2 pr-3 text-stone-600">{s.from} → {s.to}</td>
                  <td className="py-2 pr-3">
                    <button onClick={() => onWhy(e.subject_id)} className="text-left hover:underline">
                      <span className="mr-2 inline-block h-2 w-2 rounded-full align-middle" style={{ background: TONE_HEX[toneOf(e.kind)] }} aria-hidden="true" />
                      {e.summary}
                    </button>
                    <span className="block text-xs text-stone-500">{e.kind.replaceAll("_", " ")} · {e.provenance}</span>
                  </td>
                  <td className="py-2 pr-3 text-stone-600">{workflowOf(e)}</td>
                  <td className="py-2 pr-3 font-mono text-xs text-stone-500">{e.model ?? "code"}{e.latency_us > 0 ? ` · ${latency(e.latency_us)}` : ""}</td>
                  <td className="py-2">{sim ? <Badge tone="violet">{sim}</Badge> : <span className="text-xs text-stone-500">internal step</span>}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {rows.length === 0 && <p className="py-6 text-center text-sm text-stone-500">No events match.</p>}
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------------------------
// Scenario lab (Firedrill)

type ExactCheck = { key: string; service: string; operation: string; comparison: string; expected: number; observed: number | null; passed: boolean | null };
type Drill = {
  drill_id: string;
  scenario: { key?: string; name: string; persona?: string; opening_message?: string; instruction?: string; criteria?: string[]; events?: { kind: string; after_turn: number }[] };
  ending_reason: string | null;
  exact_checks: ExactCheck[];
  rubric: { name: string; status: string; score: number | null; threshold: number }[];
  turns: { user: string | null; agent: string | null }[];
};
type Round = {
  schema?: string; round: string; suite?: string | null; suite_title?: string | null; variant?: string | null; campaign_id: string; state: string; collected_at: string; payments: string;
  metrics?: { drills: number; passed: number; unsafe_actions: number; rubric_scored: number; rubric_mean: number | null; turns: number };
  drills: Drill[];
};
type RoundNotes = { schema: string; rule: string; rounds: Record<string, { variant: string; change: string }> };

const drillPassed = (d: Drill) => d.exact_checks.length > 0 && d.exact_checks.every((c) => c.passed);
/** Firedrill could not finish the session (for example its simulated person timed out), so nothing was checked. */
const notChecked = (d: Drill) => d.exact_checks.length === 0 || d.exact_checks.some((c) => c.passed === null);
/** "round-1-suite-a" -> "round-1" */
const roundKey = (r: Round) => r.round.replace(/-suite-.*$/, "");

function DrillCard({ d }: { d: Drill }) {
  const passed = drillPassed(d);
  const outage = (d.scenario.events ?? []).some((e) => e.kind === "dependency_unavailable");
  return (
    <article className="rounded-2xl border border-line bg-surface p-4">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h3 className="text-base font-semibold">{d.scenario.name}</h3>
          <p className="text-xs text-stone-500">Simulated flatmate: {d.scenario.persona ?? "—"}{outage ? " · payments outage injected" : ""}</p>
        </div>
        <Badge tone={notChecked(d) ? "ember" : passed ? "sage" : "clay"}>{notChecked(d) ? "? Not checked: Firedrill could not finish this session" : passed ? "✓ Passed" : "✕ Failed"}</Badge>
      </div>
      <dl className="mt-3 grid gap-3 sm:grid-cols-3">
        <div className="rounded-xl border border-line bg-ink/40 p-3">
          <dt className="text-[10px] uppercase tracking-widest text-stone-500">Input</dt>
          <dd className="mt-1 text-sm">“{d.scenario.opening_message ?? d.turns[0]?.user ?? ""}”</dd>
        </div>
        <div className="rounded-xl border border-line bg-ink/40 p-3">
          <dt className="text-[10px] uppercase tracking-widest text-stone-500">Expected</dt>
          <dd className="mt-1 text-sm">{d.exact_checks.map((c) => `${c.expected} × ${c.service} ${c.operation}`).join("; ")}</dd>
        </div>
        <div className="rounded-xl border border-line bg-ink/40 p-3">
          <dt className="text-[10px] uppercase tracking-widest text-stone-500">Actual</dt>
          <dd className="mt-1 text-sm">{d.exact_checks.map((c) => `${c.observed ?? "not counted"} × ${c.service} ${c.operation}`).join("; ")}</dd>
        </div>
      </dl>
      <div className="mt-3 flex flex-wrap gap-1.5 text-[11px]">
        <Badge tone="violet">SIMULATED USER</Badge>
        {Array.from(new Set(d.exact_checks.map((c) => c.service))).map((sv) => <Badge key={sv} tone="violet">SYNTHETIC {sv.toUpperCase()}</Badge>)}
        <Badge>Firedrill</Badge>
        <Badge>Superlinked SIE</Badge>
        <Badge>SingleStore</Badge>
      </div>
      {d.rubric.some((r) => r.score !== null) && <p className="mt-3 text-xs text-stone-500">Written criteria scored {d.rubric.filter((r) => r.score !== null).map((r) => `${r.score}`).join(", ")} of 100 by Firedrill’s rubric (threshold {d.rubric[0]?.threshold}).</p>}
      {d.rubric.some((r) => r.status === "not_assessable") && (
        <p className="mt-3 text-xs text-stone-500">Written criteria were not scored in this round: the evaluation asked for evidence this driver does not record. The charge count above is exact and does not depend on a model.</p>
      )}
      <details className="mt-3">
        <summary className="cursor-pointer text-sm font-medium text-teal-800">Conversation ({d.turns.length} turn{d.turns.length === 1 ? "" : "s"}) and criteria</summary>
        <ol className="mt-2 space-y-2">
          {d.turns.map((t, i) => (
            <li key={i} className="text-sm">
              <p><b className="text-stone-600">Flatmate:</b> {t.user}</p>
              <p><b className="text-sage">Stocked:</b> {t.agent}</p>
            </li>
          ))}
        </ol>
        {d.scenario.criteria && (
          <ul className="mt-3 list-disc space-y-1 pl-5 text-xs text-stone-500">
            {d.scenario.criteria.map((c) => <li key={c}>{c}</li>)}
          </ul>
        )}
      </details>
    </article>
  );
}

export function ScenarioLab() {
  const [rounds, setRounds] = useState<Round[] | null>(null);
  const [notes, setNotes] = useState<RoundNotes | null>(null);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [open, setOpen] = useState<string | null>(null);
  useEffect(() => {
    api<{ reports: { name: string; report: Round & RoundNotes }[] }>("/api/firedrill/reports")
      .then((r) => {
        setRounds(r.reports.filter((x) => x.report?.schema === "stocked.firedrill-round.v1").map((x) => x.report));
        setNotes(r.reports.find((x) => x.report?.schema === "stocked.firedrill-rounds.v1")?.report ?? null);
      })
      .catch(() => setRounds([]));
    api<{ sessions: Session[] }>("/api/sessions").then((r) => setSessions(r.sessions)).catch(() => setSessions([]));
  }, []);

  const all = rounds ?? [];
  const keys = Array.from(new Set(all.map(roundKey))).sort();
  // One row per scenario, one column per round.
  const scenarios = Array.from(new Map(all.flatMap((r) => r.drills.map((d) => [d.scenario.key ?? d.scenario.name, { key: d.scenario.key ?? d.scenario.name, name: d.scenario.name, suite: r.suite_title ?? "" }] as const))).values());
  const cell = (scenario: string, rk: string) => all.filter((r) => roundKey(r) === rk).flatMap((r) => r.drills).find((d) => (d.scenario.key ?? d.scenario.name) === scenario);
  const totals = keys.map((rk) => {
    const ds = all.filter((r) => roundKey(r) === rk).flatMap((r) => r.drills);
    const scored = ds.flatMap((d) => d.rubric).filter((x) => x.score !== null);
    return {
      rk,
      drills: ds.filter((d) => !notChecked(d)).length,
      unchecked: ds.filter(notChecked).length,
      passed: ds.filter(drillPassed).length,
      unsafe: ds.reduce((n, d) => n + d.exact_checks.reduce((m, c) => m + Math.max(0, (c.observed ?? 0) - c.expected), 0), 0),
      turns: ds.reduce((n, d) => n + d.turns.length, 0),
      rubric: scored.length ? scored.reduce((n, x) => n + (x.score ?? 0), 0) / scored.length : null,
      scored: scored.length,
    };
  });
  // The decision rule, applied to scenarios that ran in both rounds.
  const decision = (i: number): { kept: boolean; why: string } | null => {
    if (i === 0) return null;
    const [prev, cur] = [keys[i - 1], keys[i]];
    const shared = scenarios.filter((s) => cell(s.key, prev) && cell(s.key, cur) && !notChecked(cell(s.key, prev)!) && !notChecked(cell(s.key, cur)!));
    if (shared.length === 0) return null;
    const regressed = shared.filter((s) => drillPassed(cell(s.key, prev)!) && !drillPassed(cell(s.key, cur)!));
    const fixed = shared.filter((s) => !drillPassed(cell(s.key, prev)!) && drillPassed(cell(s.key, cur)!));
    const kept = regressed.length === 0 && totals[i].unsafe === 0;
    return { kept, why: kept ? `${fixed.length} scenario${fixed.length === 1 ? "" : "s"} fixed, none broken, no unwanted charges` : regressed.length ? `${regressed.length} scenario(s) that passed before now fail` : "an unwanted charge happened" };
  };
  const selected = open ? all.flatMap((r) => r.drills).find((d) => d.drill_id === open) : null;
  const latest = totals[totals.length - 1];

  return (
    <div className="space-y-4">
      <Card title="Scenario lab" hint="Firedrill">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <p className="max-w-2xl text-sm leading-relaxed text-stone-700">
            Simulated flatmates talk to Stocked through the same endpoint and the same rules a real person uses. Payments go to a synthetic Stripe, so “how many times did it charge?” is counted, not judged. Each round runs the same scenarios against a changed agent.
          </p>
          <div className="text-right">
            <div className="text-3xl font-semibold tabular-nums" style={{ color: latest && latest.passed === latest.drills ? TONE_HEX.sage : TONE_HEX.clay }}>{rounds === null ? "…" : latest ? `${latest.passed} / ${latest.drills}` : "0 / 0"}</div>
            <div className="text-xs text-stone-500">charge checks passed in the latest round</div>
            <a href="https://app.firedrill.run/simulations" target="_blank" rel="noreferrer" className="mt-2 inline-block text-sm font-medium underline decoration-stone-400 underline-offset-2 hover:text-ivory">See all simulations in Firedrill ↗</a>
          </div>
        </div>
      </Card>

      {rounds?.length === 0 && <Card><p className="text-sm text-stone-500">No collected rounds yet. Run <code className="font-mono">python3 firedrill/suites.py run A round-0 v0</code>.</p></Card>}

      {keys.length > 0 && (
        <div className="grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
          {totals.map((t, i) => {
            const d = decision(i);
            const note = notes?.rounds[t.rk];
            return (
              <Card key={t.rk}>
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h2 className="text-base font-semibold capitalize">{t.rk.replace("-", " ")}</h2>
                  <div className="flex gap-1.5">
                    <Badge>agent {note?.variant ?? all.find((r) => roundKey(r) === t.rk)?.variant ?? "?"}</Badge>
                    {d && <Badge tone={d.kept ? "sage" : "clay"}>{d.kept ? "Kept" : "Switched back"}</Badge>}
                    {!d && i === 0 && <Badge>Baseline</Badge>}
                  </div>
                </div>
                {note && <p className="mt-2 text-sm leading-relaxed text-stone-700">{note.change}</p>}
                <dl className="mt-3 grid grid-cols-4 gap-2 text-center">
                  {([["Passed", `${t.passed}/${t.drills}`], ["Unwanted charges", String(t.unsafe)], ["Turns", String(t.turns)], ["Rubric", t.rubric === null ? "—" : `${t.rubric.toFixed(0)}`]] as const).map(([k, v]) => (
                    <div key={k} className="rounded-lg border border-line bg-ink/40 px-1 py-2">
                      <dd className="text-lg font-semibold tabular-nums">{v}</dd>
                      <dt className="text-[10px] uppercase tracking-wider text-stone-500">{k}</dt>
                    </div>
                  ))}
                </dl>
                {d && <p className="mt-2 text-xs text-stone-500">Decision: {d.why}.</p>}
                {t.unchecked > 0 && <p className="mt-1 text-xs text-amber-700">{t.unchecked} session{t.unchecked === 1 ? " was" : "s were"} not checked: Firedrill could not finish {t.unchecked === 1 ? "it" : "them"}.</p>}
                {t.rubric !== null && <p className="mt-1 text-[11px] text-stone-500">Rubric is the mean of {t.scored} model-scored criteria (0–100); the rest were not scored.</p>}
              </Card>
            );
          })}
        </div>
      )}
      {notes && keys.length > 1 && <p className="px-1 text-xs text-stone-500">Rule: {notes.rule}</p>}

      {scenarios.length > 0 && (
        <Card title="Scenarios × rounds" hint="select a cell to read the session">
          <div className="overflow-x-auto">
            <table className="w-full min-w-[560px] text-left text-sm">
              <thead className="text-[11px] uppercase tracking-widest text-stone-500">
                <tr>
                  <th className="py-2 pr-3 font-semibold">Scenario</th>
                  {keys.map((k) => <th key={k} className="py-2 pr-3 font-semibold">{k.replace("-", " ")}</th>)}
                </tr>
              </thead>
              <tbody>
                {scenarios.map((s) => (
                  <tr key={s.key} className="border-t border-line">
                    <td className="py-2 pr-3">
                      <span className="font-medium">{s.name}</span>
                      <span className="block text-xs text-stone-500">{s.suite}</span>
                    </td>
                    {keys.map((k) => {
                      const d = cell(s.key, k);
                      if (!d) return <td key={k} className="py-2 pr-3 text-xs text-stone-400">not run</td>;
                      const ok = drillPassed(d);
                      const c = d.exact_checks[0];
                      return (
                        <td key={k} className="py-2 pr-3">
                          <button onClick={() => setOpen(open === d.drill_id ? null : d.drill_id)} aria-pressed={open === d.drill_id} className={`rounded-lg border px-2.5 py-1.5 text-left ${open === d.drill_id ? "border-ember" : "border-line hover:bg-raised"}`}>
                            <span className={`font-semibold ${notChecked(d) ? "text-amber-700" : ok ? "text-emerald-700" : "text-red-700"}`}>{notChecked(d) ? "? Not checked" : ok ? "✓ Passed" : "✕ Failed"}</span>
                            {c && !notChecked(d) && <span className="block text-xs text-stone-500">{d.exact_checks.length} tool-call check{d.exact_checks.length === 1 ? "" : "s"} · {d.turns.length} turn{d.turns.length === 1 ? "" : "s"}</span>}
                          </button>
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {selected && <DrillCard d={selected} />}

      <Card title="Agent sessions" hint="every tool call, with input, output and time">
        {sessions.length === 0 && <p className="text-sm text-stone-500">No conversations recorded yet.</p>}
        <div className="space-y-2">{sessions.map((s) => <SessionCard key={s.session} s={s} />)}</div>
      </Card>
    </div>
  );
}
