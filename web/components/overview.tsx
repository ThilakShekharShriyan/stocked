"use client";

// Panels around the agent world: what Stocked is doing, details for the selected node,
// and the activity stream. Everything is derived from recorded data.

import { useEffect, useState } from "react";
import { AgentEvent, House, api, dayLabel, eventSummary, latency, money, timeLabel, verified } from "@/lib/api";
import { TONE_HEX, WorldNode, autonomy, routeOf, senderOf, simulationLabel, toneOf } from "@/lib/world";

export function Card({ title, hint, children, className }: { title?: string; hint?: string; children: React.ReactNode; className?: string }) {
  return (
    <section className={`rounded-2xl border border-line bg-surface p-4 ${className ?? ""}`}>
      {title && (
        <div className="mb-3 flex items-baseline justify-between gap-3">
          <h2 className="text-[11px] font-semibold uppercase tracking-[0.14em] text-stone-500">{title}</h2>
          {hint && <span className="text-[11px] text-stone-400">{hint}</span>}
        </div>
      )}
      {children}
    </section>
  );
}

export function Badge({ tone = "ivory", children }: { tone?: "ivory" | "sage" | "ember" | "clay" | "mist" | "violet"; children: React.ReactNode }) {
  const cls = {
    ivory: "border-line text-stone-600",
    sage: "border-emerald-600/40 bg-emerald-100 text-emerald-900",
    ember: "border-amber-500/40 bg-amber-100 text-amber-900",
    clay: "border-red-600/40 bg-red-100 text-red-900",
    mist: "border-sky-800/30 bg-sky-100 text-sky-800",
    violet: "border-violet-900/30 bg-violet-100 text-violet-900",
  }[tone];
  return <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-[11px] font-semibold tracking-wide ${cls}`}>{children}</span>;
}

/** What needs a person right now, or the most recent decision if nothing does. */
export function NowPanel({ house, onWhy, onOpenPhone }: { house: House; onWhy: (id: string) => void; onOpenPhone: (member: string) => void }) {
  const held = house.plans.find((p) => p.status === "held" || p.status === "payment_failed");
  const approver = house.members.find((m) => m.role === "approver");
  const unbooked = house.outings.find((o) => o.status === "planned");
  const last = house.plans.find((p) => p.status !== "held");
  return (
    <Card title="What Stocked is doing" hint="live">
      {held ? (
        <div className="rise">
          <Badge tone="ember">Needs a person</Badge>
          <p className="mt-2 text-lg font-semibold leading-snug">{held.title}</p>
          <p className="mt-1 text-sm text-stone-600">
            {money(held.total_cents)} · {held.basket.length} items. {held.verdict.reasons[0]}.
          </p>
          <p className="mt-2 text-sm text-stone-500">
            Waiting for {approver?.name ?? "the approver"}
            {approver && !verified(house, approver.id) ? ", who first confirms they are a real person with Persona." : " to approve."}
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            {approver && (
              <button onClick={() => onOpenPhone(approver.id)} className="rounded-lg bg-ember px-3 py-1.5 text-sm font-semibold text-ink hover:brightness-110">
                Open {approver.name}’s phone
              </button>
            )}
            <button onClick={() => onWhy(held.id)} className="rounded-lg border border-line px-3 py-1.5 text-sm font-medium hover:bg-raised">
              See the evidence
            </button>
          </div>
        </div>
      ) : (
        <div>
          <Badge tone="sage">Nothing is waiting on anyone</Badge>
          {last && (
            <>
              <p className="mt-2 text-lg font-semibold leading-snug">{last.title}</p>
              <p className="mt-1 text-sm text-stone-600">
                {last.status === "ordered" ? `Ordered in test mode · ${money(last.total_cents)}` : last.decision_note}
              </p>
              <button onClick={() => onWhy(last.id)} className="mt-3 rounded-lg border border-line px-3 py-1.5 text-sm font-medium hover:bg-raised">
                See the evidence
              </button>
            </>
          )}
        </div>
      )}
      {unbooked && (
        <p className="mt-4 border-t border-line pt-3 text-sm text-stone-600">
          Also planned: <b className="text-ivory">{unbooked.title}</b> for {unbooked.quote.headcount} at {money(unbooked.quote.per_person_cents)} each, not booked yet.
        </p>
      )}
    </Card>
  );
}

export function AutonomySummary({ house }: { house: House }) {
  const a = autonomy(house);
  const rows: [string, number, "sage" | "ember" | "clay" | "ivory" | "mist"][] = [
    ["Handled automatically", a.automatic, "sage"],
    ["Waiting for approval", a.waiting, "ember"],
    ["Approved by a person", a.approvedByPerson, "mist"],
    ["Refused by a rule", a.refused, "clay"],
    ["Test-mode orders", a.testOrders, "ivory"],
    ["Test-mode bookings", a.testBookings, "ivory"],
  ];
  return (
    <Card title="Autonomy" hint="since the last reset">
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3">
        {rows.map(([label, n, tone]) => (
          <div key={label}>
            <dd className="text-2xl font-semibold tabular-nums" style={{ color: n === 0 ? undefined : TONE_HEX[tone] }}>
              {n}
            </dd>
            <dt className="text-xs text-stone-500">{label}</dt>
          </div>
        ))}
      </dl>
      <div className="mt-4 flex items-center justify-between rounded-xl border border-line bg-raised px-3 py-2">
        <span className="text-sm">Real-world actions</span>
        <span className="text-xl font-semibold tabular-nums">{a.realWorldActions}</span>
      </div>
      <p className="mt-1.5 text-[11px] text-stone-500">Every order and booking runs in test mode. No real money moves and nothing real is reserved.</p>
    </Card>
  );
}

type Fact = [label: string, value: string];
type RoundReport = { schema?: string; round: string; variant?: string | null; drills: { exact_checks: { passed: boolean | null }[] }[] };

/** What each system has actually done for this flat, from recorded data. */
function useSystemFacts(node: WorldNode, house: House, events: AgentEvent[]): { facts: Fact[]; related: AgentEvent[]; note?: string } {
  const [rounds, setRounds] = useState<RoundReport[] | null>(null);
  useEffect(() => {
    if (node.id !== "firedrill") return;
    api<{ reports: { report: RoundReport }[] }>("/api/firedrill/reports").then((r) => setRounds(r.reports.map((x) => x.report).filter((x) => x.schema === "stocked.firedrill-round.v1"))).catch(() => setRounds([]));
  }, [node.id]);
  const t = house.totals;
  const of = (...kinds: string[]) => events.filter((e) => kinds.includes(e.kind));
  const world = (service: string) => events.filter((e) => e.kind === "world_call" && e.data.service === service);
  switch (node.id) {
    case "sie": case "reader": case "classifier": {
      const mine = events.filter((e) => e.model && (node.id === "sie" || (node.id === "reader" ? e.kind === "line_extracted" : e.kind === "decision_gate")));
      const byModel = new Map<string, { n: number; us: number }>();
      for (const e of mine) { const m = byModel.get(e.model!) ?? { n: 0, us: 0 }; m.n += 1; m.us += e.latency_us; byModel.set(e.model!, m); }
      const facts: Fact[] = Array.from(byModel, ([m, v]) => [m, `${v.n} call${v.n === 1 ? "" : "s"} · avg ${latency(Math.round(v.us / v.n))}`]);
      if (node.id === "sie") facts.push(["All model steps recorded", `${t.model_steps} · ${t.model_credits.toFixed(0)} credits`]);
      return { facts, related: mine.slice(0, 6), note: facts.length ? undefined : "No model calls in the latest events." };
    }
    case "singlestore": case "audit": {
      const d = house.database;
      const facts: Fact[] = d.tables.map((x) => [x.table.replaceAll("_", " "), `${x.here} here · ${x.all} in total`]);
      facts.push(["store catalog (products)", String(d.catalog_products)], ["households stored", `${d.households} incl. simulated`]);
      if (node.id === "audit") return { facts: d.event_kinds.map((k) => [k.kind.replaceAll("_", " "), String(k.n)] as Fact), related: events.slice(0, 12), note: "Audit events for this flat by type. Append-only: rows are never updated or deleted." };
      return { facts, related: events.slice(0, 12), note: "The only database. Run-out dates are computed in SQL from purchases; an order and its audit row are written together, and the order table is keyed on the idempotency key so a retry cannot charge twice. “In total” includes the test households Firedrill simulations create." };
    }
    case "airbyte": case "inbox": case "calendars": {
      const syncs = of("airbyte_sync");
      return { facts: [["Sync runs (latest events)", String(syncs.length)], ["Receipts synced", `${t.receipts_synced} of ${t.receipts} (the rest are seeded history)`], ["Calendar events synced", String(house.calendar.filter((c) => c.source === "airbyte").length)], ["Last sync took", syncs[0] ? latency(syncs[0].latency_us) : "—"]], related: [...syncs, ...of("receipt_ingested", "invite_received")].slice(0, 6), note: "Source: Airbyte’s open-source file connector reading fictional data files." };
    }
    case "statsig": {
      const versions = Array.from(new Set(house.plans.map((p) => p.agent_version)));
      return { facts: [["Connected", t.statsig_connected ? "yes, rules read live" : "no, using defaults"], ["Auto-approve limit", money(house.budget.auto_limit_cents)], ["Planner variant now", house.budget.agent_version], ["Variants seen on plans", versions.join(", ") || "—"]], related: of("plan_auto_approved", "plan_held").slice(0, 6), note: "Gates: stocked_model_planner and stocked_contingency. A gate that has not been created in the Statsig console reads as off." };
    }
    case "amplitude":
      return { facts: [["Configured", t.amplitude_configured ? "yes" : "no"], ["Events sent", `${t.events_forwarded} of ${t.events}`], ["Waiting to send", String(t.events - t.events_forwarded)], ["Conversation turns (latest)", String(of("agent_reply").length)]], related: of("chat_message", "agent_reply", "tool_call").slice(0, 6), note: "Sent as Agent Analytics events (user message, tool call, AI response, score) plus plan and order outcomes, tagged with the agent version." };
    case "firedrill": {
      if (rounds === null) return { facts: [], related: [], note: "Loading simulation results…" };
      const drills = rounds.flatMap((r) => r.drills);
      const checked = drills.filter((d) => d.exact_checks.length > 0 && d.exact_checks.every((c) => c.passed !== null));
      const names = Array.from(new Set(rounds.map((r) => r.round.replace(/-suite-.*$/, "")))).sort();
      return { facts: [["Rounds run", names.length ? names.map((n) => n.replace("-", " ")).join(", ") : "none"], ["Simulated sessions", String(drills.length)], ["Passed every tool-call check", `${checked.filter((d) => d.exact_checks.every((c) => c.passed)).length} of ${checked.length}`], ["Could not be finished by Firedrill", String(drills.length - checked.length)]], related: [], note: "Simulations run in their own test households, so they do not appear in this flat’s activity. Open the Scenario lab for every session." };
    }
    case "payments":
      return { facts: [["Provider", house.payments.provider.replaceAll("_", " ")], ["Test-mode charges", String(house.orders.length)], ["Total charged (test)", money(house.orders.reduce((n, o) => n + o.total_cents, 0))], ["Real payments", "0"]], related: of("order_placed", "duplicate_prevented", "payment_failed", "booking_changed", "replanned_to_backup").slice(0, 6) };
    case "webworld": case "gcal": case "mailbox": {
      const calls = world(node.id === "gcal" ? "calendar" : node.id === "mailbox" ? "mail" : "web");
      return { facts: [["Mode", house.world.mode === "firedrill-synthetic" ? "Firedrill synthetic" : "built-in stand-in (SIMULATED)"], ["Calls in the latest events", String(calls.length)]], related: calls.slice(0, 6), note: calls.length ? undefined : "Nothing yet in this flat. Use the “Outside world” buttons on an outing, or see the Scenario lab for runs against Firedrill’s synthetic services." };
    }
    default:
      return { facts: [], related: [] };
  }
}

/** Details for whatever is selected in the scene. */
export function NodeDetails({ node, house, events, onWhy, onClose }: { node: WorldNode; house: House; events: AgentEvent[]; onWhy: (id: string) => void; onClose: () => void }) {
  const memberId = node.kind === "person" ? node.id.slice(2) : null;
  const member = house.members.find((m) => m.id === memberId);
  const sys = useSystemFacts(node, house, events);
  const routed = events.filter((e) => { const r = routeOf(e, house); return r && (r.from === node.id || r.to === node.id); }).slice(0, 6);
  const involved = sys.related.length ? sys.related : routed;
  const balance = house.balances.find((b) => b.member_id === memberId)?.net_cents ?? 0;
  const proof = memberId ? verified(house, memberId) : undefined;
  return (
    <Card className="rise">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.14em] text-ember">{node.kind === "checkpoint" ? "Human checkpoint" : node.kind}</div>
          <h2 className="text-xl font-semibold">{node.label}</h2>
          <p className="text-sm text-stone-600">{node.role}</p>
        </div>
        <button onClick={onClose} className="rounded-lg border border-line px-2.5 py-1 text-xs font-medium hover:bg-raised">Close</button>
      </div>
      <p className="mt-3 text-sm leading-relaxed text-stone-700">{node.detail}</p>
      {node.poweredBy && <p className="mt-2 text-xs text-stone-500">Runs on: <span className="font-mono text-stone-600">{node.poweredBy}</span></p>}

      {member && (
        <div className="mt-4 space-y-3 border-t border-line pt-3">
          <div className="flex flex-wrap gap-2">
            {member.role === "approver" && <Badge tone="ember">House approver</Badge>}
            {proof ? <Badge tone="sage">Live human confirmed{proof.provider === "simulated" ? " · SIMULATED" : " · SANDBOX"}</Badge> : <Badge>Not verified right now</Badge>}
            <Badge>{balance === 0 ? "Square" : balance > 0 ? `Owed ${money(balance)}` : `Owes ${money(-balance)}`}</Badge>
          </div>
          <div>
            <h3 className="mb-1 text-xs font-semibold uppercase tracking-widest text-stone-500">On their calendar</h3>
            {house.calendar.filter((c) => c.host_member_id === member.id).length === 0 && <p className="text-sm text-stone-500">Nothing in the synced window.</p>}
            <ul className="space-y-1 text-sm">
              {house.calendar.filter((c) => c.host_member_id === member.id).map((c) => (
                <li key={c.id}><b>{c.title}</b> · {dayLabel(c.starts_at)} · {eventSummary(c)}</li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="mb-1 text-xs font-semibold uppercase tracking-widest text-stone-500">What their agent answered</h3>
            {house.outings.length === 0 && <p className="text-sm text-stone-500">No outings planned.</p>}
            <ul className="space-y-1.5 text-sm">
              {house.outings.map((o) => {
                const i = o.invitees.find((x) => x.member_id === member.id);
                if (!i) return null;
                return (
                  <li key={o.id}>
                    <b>{o.title}</b>: {i.status === "going" ? "in" : "out"} — {i.reason}
                    <span className="block text-xs text-stone-500">Spending limit {money(i.cap_cents)} · plan is {money(o.quote.per_person_cents)} each{i.by !== "agent" ? " · changed by them" : ""}</span>
                  </li>
                );
              })}
            </ul>
          </div>
        </div>
      )}

      {node.kind === "checkpoint" && (
        <div className="mt-4 border-t border-line pt-3 text-sm">
          <p className="mb-2 text-stone-600">Provider: {house.identity.provider === "persona_relay" ? "Persona Relay" : house.identity.provider === "persona" ? "Persona" : "none connected (checks are SIMULATED)"}{house.identity.sandbox ? " · SANDBOX" : ""}. A check is valid for {house.identity.valid_minutes} minutes.</p>
          <ul className="space-y-1">
            {house.verifications.slice(0, 4).map((v) => (
              <li key={v.id} className="flex justify-between gap-2"><span>{house.members.find((m) => m.id === v.member_id)?.name}</span><span className="text-stone-500">{v.status}{v.provider === "simulated" ? " · SIMULATED" : ""}</span></li>
            ))}
            {house.verifications.length === 0 && <li className="text-stone-500">No checks requested yet.</li>}
          </ul>
        </div>
      )}

      {(sys.facts.length > 0 || sys.note) && (
        <div className="mt-4 border-t border-line pt-3">
          <h3 className="mb-2 text-xs font-semibold uppercase tracking-widest text-stone-500">What it has done here</h3>
          <dl className="space-y-1.5">
            {sys.facts.map(([k, v]) => (
              <div key={k} className="flex items-baseline justify-between gap-3 text-sm">
                <dt className="min-w-0 truncate text-stone-600" title={k}>{k}</dt>
                <dd className="shrink-0 text-right font-semibold tabular-nums">{v}</dd>
              </div>
            ))}
          </dl>
          {sys.note && <p className="mt-2 text-xs leading-relaxed text-stone-500">{sys.note}</p>}
          {node.id === "firedrill" && (
            <a href="https://app.firedrill.run/simulations" target="_blank" rel="noreferrer" className="mt-3 inline-flex items-center gap-1.5 rounded-lg bg-ember px-3 py-1.5 text-sm font-semibold text-ink hover:opacity-90">
              See all simulations in Firedrill <span aria-hidden>↗</span><span className="sr-only">(opens in a new tab)</span>
            </a>
          )}
        </div>
      )}

      <div className="mt-4 border-t border-line pt-3">
        <h3 className="mb-2 text-xs font-semibold uppercase tracking-widest text-stone-500">Recent activity here</h3>
        {involved.length === 0 && <p className="text-sm text-stone-500">{node.id === "firedrill" ? "See the Scenario lab." : "Nothing recorded in the latest events."}</p>}
        <ul className="space-y-2">
          {involved.map((e) => (
            <li key={e.id} className="text-sm">
              <button onClick={() => onWhy(e.subject_id)} className="text-left hover:underline">{e.summary}</button>
              <span className="block text-xs text-stone-500">{timeLabel(e.ts)}{e.model ? ` · ${e.model}` : ""}{e.latency_us > 0 ? ` · ${latency(e.latency_us)}` : ""}</span>
            </li>
          ))}
        </ul>
      </div>
    </Card>
  );
}

/** Who told whom what, in order, with a link to the evidence. */
export function ActivityStream({ house, nodes, events, onWhy, onSelectNode, limit = 14 }: { house: House; nodes: WorldNode[]; events: AgentEvent[]; onWhy: (id: string) => void; onSelectNode?: (id: string) => void; limit?: number }) {
  const shown = events.filter((e) => e.kind !== "turn_score").slice(0, limit);
  return (
    <ol className="space-y-1.5">
      {shown.length === 0 && <li className="text-sm text-stone-500">No activity yet. Press “Sync inbox” to bring in receipts and calendars.</li>}
      {shown.map((e) => {
        const s = senderOf(e, house, nodes);
        const r = routeOf(e, house);
        const sim = simulationLabel(e);
        const tone = toneOf(e.kind);
        return (
          <li key={e.id} className="grid grid-cols-[auto_1fr_auto] items-start gap-x-3 rounded-lg px-2 py-1.5 hover:bg-raised">
            <span className="mt-1.5 h-2 w-2 rounded-full" style={{ background: TONE_HEX[tone] }} aria-hidden="true" />
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-x-1.5 text-[11px] text-stone-500">
                <button disabled={!r || !onSelectNode} onClick={() => r && onSelectNode?.(r.from)} className="font-semibold text-stone-700 enabled:hover:underline">{s.from}</button>
                <span aria-hidden="true">→</span>
                <button disabled={!r || !onSelectNode} onClick={() => r && onSelectNode?.(r.to)} className="font-semibold text-stone-700 enabled:hover:underline">{s.to}</button>
                {sim && <Badge tone="violet">{sim}</Badge>}
                {e.provenance === "observed" && <span>· observed</span>}
              </div>
              <button onClick={() => onWhy(e.subject_id)} className="block w-full truncate text-left text-sm hover:underline" title={e.summary}>{e.summary}</button>
            </div>
            <span className="whitespace-nowrap text-[11px] tabular-nums text-stone-500">{timeLabel(e.ts)}</span>
          </li>
        );
      })}
    </ol>
  );
}
