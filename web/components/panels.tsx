"use client";

// The three home panels: Running low · Coming up · Ledger. Every row opens the Why drawer.

import { House, Item, Plan, covered, dayLabel, eventSummary, money, nameOf, timeLabel } from "@/lib/api";

export function Panel({ title, hint, children }: { title: string; hint?: string; children: React.ReactNode }) {
  return (
    <section className="rounded-2xl border border-stone-200 bg-white p-5 shadow-sm">
      <div className="mb-4 flex items-baseline justify-between gap-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-stone-500">{title}</h2>
        {hint && <span className="text-xs text-stone-400">{hint}</span>}
      </div>
      {children}
    </section>
  );
}

const STATUS: Record<string, { label: string; bar: string; text: string }> = {
  low: { label: "Running out", bar: "bg-red-600", text: "text-red-700" },
  soon: { label: "Soon", bar: "bg-amber-500", text: "text-amber-700" },
  ok: { label: "Stocked", bar: "bg-emerald-600", text: "text-emerald-700" },
};

function DaysBar({ item }: { item: Item }) {
  const s = STATUS[item.status] ?? STATUS.ok;
  const total = Math.max(item.interval_days ?? 14, 1);
  const pct = Math.max(4, Math.min(100, ((item.days_left ?? 0) / total) * 100));
  return (
    <div className="mt-2 h-2 w-full rounded-full bg-stone-100" role="img" aria-label={`${s.label}: about ${Math.round(item.days_left ?? 0)} days left`}>
      <div className={`h-2 rounded-full ${s.bar}`} style={{ width: `${pct}%` }} />
    </div>
  );
}

export function RunningLow({ house, onWhy }: { house: House; onWhy: (id: string) => void }) {
  const tracked = house.items.filter((i) => i.status === "low" || i.status === "soon" || i.status === "ok");
  const shown = tracked.slice(0, 7);
  const cov = covered(house);
  return (
    <Panel title="Running low" hint="from receipts · nobody typed this">
      <ul className="space-y-3">
        {shown.map((i) => {
          const s = STATUS[i.status] ?? STATUS.ok;
          return (
            <li key={i.id}>
              <button onClick={() => onWhy(i.id)} className="w-full rounded-lg p-2 text-left hover:bg-stone-50">
                <div className="flex items-baseline justify-between gap-2">
                  <span className="text-base font-medium">{i.name}</span>
                  <span className={`text-sm font-semibold ${s.text}`}>
                    {s.label} · {i.days_left !== null && i.days_left < 0.5 ? "now" : dayLabel(i.runs_out_on)}
                  </span>
                </div>
                <DaysBar item={i} />
                <div className="mt-1 text-xs text-stone-500">
                  {i.basis}
                  {cov.has(i.id) && <span className="ml-2 rounded bg-emerald-100 px-1.5 py-0.5 font-medium text-emerald-900">{cov.get(i.id)}</span>}
                  {i.provenance === "observed" && <span className="ml-2 rounded bg-sky-100 px-1.5 py-0.5 font-medium text-sky-800">seen by a flatmate</span>}
                </div>
              </button>
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}

const PLAN_STATUS: Record<string, { label: string; cls: string }> = {
  held: { label: "Waiting for approval", cls: "bg-amber-100 text-amber-900" },
  ordered: { label: "Ordered", cls: "bg-emerald-100 text-emerald-900" },
  auto_approved: { label: "Auto-approved", cls: "bg-emerald-100 text-emerald-900" },
  approved: { label: "Approved", cls: "bg-emerald-100 text-emerald-900" },
  refused: { label: "Refused", cls: "bg-red-100 text-red-900" },
  skipped: { label: "Skipped", cls: "bg-stone-200 text-stone-800" },
  nothing_to_buy: { label: "Nothing to buy", cls: "bg-stone-200 text-stone-800" },
  payment_failed: { label: "Payment failed · not charged", cls: "bg-red-100 text-red-900" },
};

export function StatusPill({ status }: { status: string }) {
  const s = PLAN_STATUS[status] ?? { label: status, cls: "bg-stone-200 text-stone-800" };
  return <span className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-semibold ${s.cls}`}>{s.label}</span>;
}

export function ComingUp({ house, onWhy }: { house: House; onWhy: (id: string) => void }) {
  const planFor = (eventId: string) => house.plans.find((p) => p.trigger_ref === eventId);
  const restocks = house.plans.filter((p) => p.kind === "restock" && p.status === "held");
  const empty = house.calendar.length === 0 && restocks.length === 0;
  return (
    <Panel title="Coming up" hint={`from ${house.members.length} calendars`}>
      {empty && <p className="text-sm text-stone-500">Nothing on the calendar that needs supplies.</p>}
      <ul className="space-y-3">
        {house.calendar.map((e) => {
          const p = planFor(e.id);
          return (
            <li key={e.id} className="rounded-xl border border-stone-200 p-3">
              <div className="flex items-baseline justify-between gap-2">
                <span className="text-base font-medium">{e.title}</span>
                <span className="text-sm text-stone-600">{dayLabel(e.starts_at)}</span>
              </div>
              <div className="text-sm text-stone-500">
                <span className="mr-1 rounded bg-teal-50 px-1.5 py-0.5 text-xs font-semibold text-teal-900">{nameOf(house.members, e.host_member_id)}’s calendar</span>
                {eventSummary(e)}
                {e.source === "simulated" && <span className="ml-2 rounded bg-stone-100 px-1.5 py-0.5 text-xs">simulated invite</span>}
                {e.source === "airbyte" && <span className="ml-2 rounded bg-stone-100 px-1.5 py-0.5 text-xs">synced by Airbyte</span>}
              </div>
              {p ? <PlanCard plan={p} onWhy={onWhy} /> : <p className="mt-2 text-sm text-stone-500">{e.kind === "away" ? "Nothing to buy." : "Planning…"}</p>}
            </li>
          );
        })}
        {restocks.map((p) => (
          <li key={p.id} className="rounded-xl border border-stone-200 p-3">
            <div className="text-base font-medium">{p.title}</div>
            <PlanCard plan={p} onWhy={onWhy} />
          </li>
        ))}
      </ul>
    </Panel>
  );
}

function PlanCard({ plan, onWhy }: { plan: Plan; onWhy: (id: string) => void }) {
  return (
    <div className="mt-3">
      <div className="mb-2 flex items-center justify-between gap-2">
        <StatusPill status={plan.status} />
        {plan.basket.length > 0 && <span className="text-base font-semibold">{money(plan.total_cents)}</span>}
      </div>
      {plan.basket.length === 0 && <p className="text-sm text-stone-600">{plan.decision_note}</p>}
      <ul className="space-y-1.5">
        {plan.basket.map((l) => (
          <li key={l.item_id} className="text-sm">
            <span className="font-medium">
              {l.name} ×{l.qty}
            </span>
            <span className={`block text-xs ${l.arrives_in_time === false ? "text-red-700" : "text-stone-500"}`}>
              {l.channel === "online" ? "🚚 " : l.channel === "instore" ? "🛒 " : ""}
              {l.sourcing_note}
            </span>
          </li>
        ))}
      </ul>
      <button onClick={() => onWhy(plan.id)} className="mt-3 text-sm font-medium text-teal-800 underline underline-offset-2">
        Why this plan?
      </button>
    </div>
  );
}

export function Ledger({ house, onWhy }: { house: House; onWhy: (id: string) => void }) {
  const decided = house.plans.filter((p) => p.status !== "held" && p.status !== "nothing_to_buy").slice(0, 7);
  return (
    <Panel title="Ledger" hint="who paid · who owes">
      <div className="mb-4 rounded-xl bg-stone-50 p-3">
        {house.settlements.length === 0 ? (
          <p className="text-sm text-stone-600">Everyone is square.</p>
        ) : (
          <ul className="space-y-1">
            {house.settlements.map((s, i) => (
              <li key={i} className="flex justify-between text-sm">
                <span>
                  <b>{nameOf(house.members, s.from)}</b> owes <b>{nameOf(house.members, s.to)}</b>
                </span>
                <span className="font-semibold">{money(s.cents)}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
      {decided.length === 0 && <p className="text-sm text-stone-500">No orders yet.</p>}
      <ul className="space-y-2">
        {decided.map((p) => (
          <li key={p.id}>
            <button onClick={() => onWhy(p.id)} className="w-full rounded-lg p-2 text-left hover:bg-stone-50">
              <div className="flex items-baseline justify-between gap-2">
                <span className="text-sm font-medium">{p.title}</span>
                <span className="text-sm font-semibold">{money(p.total_cents)}</span>
              </div>
              <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-stone-500">
                <StatusPill status={p.status} />
                {p.decided_by && (
                  <span>
                    {p.decided_by === "agent" ? "by Stocked" : `by ${nameOf(house.members, p.decided_by)}`} · {timeLabel(p.decided_at)}
                  </span>
                )}
                {p.status === "ordered" && <span className="rounded bg-violet-100 px-1.5 py-0.5 font-semibold text-violet-900">TEST MODE</span>}
              </div>
              {(p.status === "refused" || p.status === "skipped") && p.decision_note && <div className="mt-1 text-xs text-red-800">{p.decision_note}</div>}
            </button>
          </li>
        ))}
      </ul>
    </Panel>
  );
}
