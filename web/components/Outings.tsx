"use client";

// Outings: who is going (answered by each person's agent), and a headcount slider that re-prices
// the plan for any group size against every person's own limit.

import { useEffect, useState } from "react";
import { House, Outing, Quote, api, dayLabel, money } from "@/lib/api";
import { Panel } from "./panels";

function RideLine({ q }: { q: Quote }) {
  const alt = q.alternatives[0];
  return (
    <div className="text-sm">
      <span className="font-semibold">
        {q.ride.mode === "cab" ? "🚕" : "🚇"} {q.ride.label} · {money(q.ride.per_person_cents)} each
      </span>
      {alt && (
        <span className="text-stone-500">
          {" "}
          instead of {alt.label.toLowerCase()} at {money(alt.per_person_cents)} each
          {alt.extra_min > 0 ? `, which takes ${alt.extra_min} min longer` : q.ride.extra_min > 0 ? `, which would be ${q.ride.extra_min} min faster` : ""}
        </span>
      )}
    </div>
  );
}

/** Things the agent does in the outside world for an outing. In the dashboard that world is a labelled stand-in. */
function WorldActions({ outing, closed, onClosed }: { outing: Outing; closed: boolean; onClosed: (c: boolean) => void }) {
  const [busy, setBusy] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const run = async (label: string, path: string, body: object, say: (r: Record<string, unknown>) => string) => {
    setBusy(label);
    setNote(null);
    try {
      setNote(say(await api<Record<string, unknown>>(path, body)));
    } catch (e) {
      setNote(e instanceof Error ? e.message : String(e));
    }
    setBusy(null);
  };
  const btn = "rounded-lg border border-line px-2.5 py-1.5 text-xs font-medium hover:bg-raised disabled:opacity-50";
  return (
    <div className="mt-3 rounded-lg border border-line bg-ink/40 p-3">
      <div className="mb-2 flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-widest text-stone-500">
        Outside world <span className="rounded-full border border-violet-900/30 bg-violet-100 px-2 py-0.5 font-semibold normal-case tracking-normal text-violet-900">SIMULATED · fictional sites</span>
      </div>
      <div className="flex flex-wrap gap-1.5">
        <button className={btn} disabled={!!busy} onClick={() => run("venue", `/api/outings/${outing.id}/check-venue`, { member: outing.organizer }, (r) => (r.open ? `The venue’s page says it is open: “${String(r.page_says)}”` : r.moved_to_backup ? `The venue’s page shows a closure notice. Moved to the backup.` : `The venue’s page shows a closure notice. Nothing was changed; the organiser needs to decide.`))}>
          {busy === "venue" ? "Reading the page…" : "Check the venue’s page"}
        </button>
        <button className={btn} disabled={!!busy || outing.status !== "booked"} onClick={() => run("ride", `/api/outings/${outing.id}/book-ride`, {}, (r) => (r.booked ? `Cab booked: ${String(r.confirmation)}` : r.already_booked ? `Already booked: ${String(r.confirmation)}` : String(r.note ?? "")))}>
          {busy === "ride" ? "Booking…" : "Arrange the ride"}
        </button>
        <button className={btn} disabled={!!busy} onClick={() => run("mail", `/api/outings/${outing.id}/send-confirmation`, {}, (r) => `Plan emailed to ${(r.to as string[]).length} people (simulated; no real email).`)}>
          {busy === "mail" ? "Sending…" : "Email the plan"}
        </button>
        <button className={`${btn} border-dashed`} disabled={!!busy} onClick={async () => { await api("/api/world/venue", { venue: outing.venue, closed: !closed }); onClosed(!closed); setNote(!closed ? `Demo: ${outing.venue}’s page now shows a closure notice. Press “Check the venue’s page”.` : `Demo: the closure notice was removed.`); }}>
          {closed ? "Demo: remove closure notice" : "Demo: venue posts a closure"}
        </button>
      </div>
      {outing.status !== "booked" && <p className="mt-2 text-[11px] text-stone-500">The ride can be arranged once the organiser has booked.</p>}
      {note && <p className="mt-2 text-sm text-stone-700" role="status">{note}</p>}
    </div>
  );
}

function OutingCard({ outing, onWhy, onWhatIf, active }: { outing: Outing; onWhy: (id: string) => void; onWhatIf?: (outingId: string, n: number) => void; active?: boolean }) {
  const [closed, setClosed] = useState(false);
  const going = outing.quote.headcount;
  const [n, setN] = useState(going);
  useEffect(() => setN(going), [going, outing.version]);
  const q = outing.what_if[n - 1] ?? outing.quote;
  // People in slider order: those going first, then the rest, then extra guests.
  const order = [...outing.invitees.filter((i) => i.status === "going"), ...outing.invitees.filter((i) => i.status !== "going")];
  const scale = Math.max(...order.map((i) => i.cap_cents), q.per_person_cents) * 1.1;
  const hypothetical = n !== going;

  return (
    <div className={`rounded-xl border p-4 ${active ? "border-ember" : "border-stone-200"}`} onClick={() => onWhatIf?.(outing.id, n)}>
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <h3 className="text-lg font-semibold">{outing.title}</h3>
        <span className="text-sm text-stone-600">
          {dayLabel(outing.starts_at)} · {outing.venue}
        </span>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-2 text-xs">
        <span className={`rounded-full px-2.5 py-0.5 font-semibold ${outing.status === "booked" ? "bg-emerald-100 text-emerald-900" : "bg-amber-100 text-amber-900"}`}>
          {outing.status === "booked" ? `Booked for ${going}` : `Planned for ${going} · not booked yet`}
        </span>
        {outing.status === "booked" && <span className="rounded bg-violet-100 px-1.5 py-0.5 font-semibold text-violet-900">TEST MODE</span>}
        <span className="text-stone-500">organised by {outing.invitees.find((i) => i.member_id === outing.organizer)?.name}</span>
      </div>

      {outing.booking?.disruption && (
        <p className="mt-3 rounded-lg border border-amber-500/40 bg-amber-100 px-3 py-2 text-sm text-amber-900" role="status">
          <b>Plan disrupted.</b> {outing.booking.disruption}
        </p>
      )}
      <p className="mt-2 text-xs text-stone-500">
        {outing.pricing.backup ? (
          <>If this falls through: <b className="text-stone-700">{outing.pricing.backup.venue}</b> at {money(outing.pricing.backup.per_head_cents)} a head before the ride. The agent switches by itself only if that costs no more than agreed.</>
        ) : (
          <>No backup venue is stored for this plan.</>
        )}
      </p>
      <WorldActions outing={outing} closed={closed} onClosed={setClosed} />
      <div className="mt-4 grid gap-5 lg:grid-cols-2">
        <div>
          <div className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-500">Who’s going · answered by each person’s agent</div>
          <ul className="space-y-1.5">
            {outing.invitees.map((i) => (
              <li key={i.member_id} className="flex gap-2 text-sm">
                <span className={`w-4 shrink-0 font-bold ${i.status === "going" ? "text-emerald-700" : "text-stone-400"}`} aria-label={i.status === "going" ? "Going" : "Not going"}>
                  {i.status === "going" ? "✓" : "–"}
                </span>
                <span>
                  <b>{i.name}</b> {i.status === "going" ? "is in" : "is out"}
                  <span className="block text-xs text-stone-500">
                    {i.reason}
                    {i.by !== "agent" && " · changed by them"}
                  </span>
                </span>
              </li>
            ))}
          </ul>
          <button onClick={() => onWhy(outing.id)} className="mt-3 text-sm font-medium text-teal-800 underline underline-offset-2">
            Why this plan?
          </button>
        </div>

        <div className="rounded-xl bg-stone-50 p-4">
          <label htmlFor={`n-${outing.id}`} className="flex items-baseline justify-between text-sm">
            <span className="font-semibold">What if we’re…</span>
            <span className="text-2xl font-semibold tabular-nums">
              {n} {n === 1 ? "person" : "people"}
            </span>
          </label>
          <input id={`n-${outing.id}`} type="range" min={1} max={outing.what_if.length} value={n} onChange={(e) => { setN(Number(e.target.value)); onWhatIf?.(outing.id, Number(e.target.value)); }} onFocus={() => onWhatIf?.(outing.id, n)} className="mt-2 w-full accent-teal-800" />
          <div className="mt-3 flex items-baseline justify-between">
            <span className="text-sm text-stone-600">Each pays</span>
            <span className="text-2xl font-semibold tabular-nums">{money(q.per_person_cents)}</span>
          </div>
          <RideLine q={q} />
          <p className="mt-1 text-xs text-stone-600">{q.why}.</p>

          <ul className="mt-4 space-y-2" aria-label="Each person's cost against their own limit">
            {order.slice(0, Math.min(n, order.length)).map((i) => {
              const over = q.over_cap.includes(i.member_id);
              return (
                <li key={i.member_id}>
                  <div className="flex justify-between text-xs">
                    <span className="font-medium">{i.name}</span>
                    <span className={over ? "font-semibold text-red-700" : "text-stone-600"}>{over ? `over their ${money(i.cap_cents)} limit` : `within ${money(i.cap_cents)} limit`}</span>
                  </div>
                  <div className="relative mt-1 h-2.5 rounded-full bg-stone-200">
                    <div className={`h-2.5 rounded-full ${over ? "bg-red-600" : "bg-teal-700"}`} style={{ width: `${Math.min(100, (q.per_person_cents / scale) * 100)}%` }} />
                    <div className="absolute -top-0.5 h-3.5 w-0.5 bg-stone-900" style={{ left: `${(i.cap_cents / scale) * 100}%` }} title={`${i.name}'s limit`} />
                  </div>
                </li>
              );
            })}
            {n > order.length && <li className="text-xs text-stone-500">+ {n - order.length} guest{n - order.length === 1 ? "" : "s"} from outside the flat</li>}
          </ul>
          <p className="mt-3 text-xs text-stone-500">
            {hypothetical ? `A what-if. The plan is still for ${going}.` : "This is the current plan."} Total {money(q.total_cents)}.
          </p>
        </div>
      </div>
    </div>
  );
}

export function Outings({ house, onWhy, onWhatIf, activeId }: { house: House; onWhy: (id: string) => void; onWhatIf?: (outingId: string, n: number) => void; activeId?: string | null }) {
  if (house.outings.length === 0) return null;
  return (
    <Panel title="Outings" hint="cost follows the headcount">
      <div className="grid gap-4">
        {house.outings.map((o) => (
          <OutingCard key={o.id} outing={o} onWhy={onWhy} onWhatIf={onWhatIf} active={activeId === o.id} />
        ))}
      </div>
    </Panel>
  );
}
