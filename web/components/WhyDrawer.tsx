"use client";

// The audit trail for one item or plan, in plain sentences:
// sources → what was proposed → checks → decision → models used.

import { useEffect, useState } from "react";
import { Check, House, Why, api, date, dayLabel, latency, money, nameOf, timeLabel } from "@/lib/api";
import { StatusPill } from "./panels";

function Highlight({ text, span }: { text: string; span: string }) {
  const i = span ? text.indexOf(span) : -1;
  if (i < 0) return <>{text}</>;
  return (
    <>
      {text.slice(0, i)}
      <mark>{span}</mark>
      {text.slice(i + span.length)}
    </>
  );
}

const CHECK_ICON: Record<Check["status"], { icon: string; label: string; cls: string }> = {
  pass: { icon: "✓", label: "Passed", cls: "text-emerald-700" },
  fail: { icon: "✕", label: "Failed", cls: "text-red-700" },
  cannot_measure: { icon: "?", label: "Cannot measure", cls: "text-amber-700" },
};

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="mb-6">
      <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-stone-500">{title}</h3>
      {children}
    </section>
  );
}

export function WhyDrawer({ subject, house, onClose }: { subject: string | null; house: House; onClose: () => void }) {
  const [why, setWhy] = useState<Why | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!subject) return;
    setWhy(null);
    setError(null);
    api<Why>(`/api/why/${subject}`).then(setWhy).catch((e) => setError(String(e.message ?? e)));
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // refetch when the house changes so the trail stays live
  }, [subject, onClose, house.recent_events[0]?.id]);

  if (!subject) return null;
  const title = why?.plan?.title ?? why?.item?.name ?? house.outings.find((o) => o.id === subject)?.title ?? "…";

  return (
    <div className="fixed inset-0 z-40 flex justify-end bg-black/60" onClick={onClose}>
      <aside
        role="dialog"
        aria-modal="true"
        aria-label={`Why: ${title}`}
        className="h-full w-full max-w-xl overflow-y-auto border-l border-line bg-surface p-6 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-6 flex items-start justify-between gap-4">
          <div>
            <div className="text-xs font-semibold uppercase tracking-wider text-teal-800">Why</div>
            <h2 className="text-2xl font-semibold">{title}</h2>
          </div>
          <button onClick={onClose} autoFocus className="rounded-lg border border-stone-300 px-3 py-1.5 text-sm font-medium hover:bg-stone-50">
            Close
          </button>
        </div>

        {error && <p className="text-sm text-red-700">{error}</p>}
        {!why && !error && <p className="text-sm text-stone-500">Loading…</p>}

        {why?.item && (
          <>
            <Section title="Estimate">
              <p className="text-base">
                {why.item.runs_out_on ? (
                  <>
                    Runs out <b>{dayLabel(why.item.runs_out_on)}</b>. {why.item.basis}.
                  </>
                ) : (
                  <>{why.item.basis}.</>
                )}
              </p>
              <p className="mt-1 text-sm text-stone-500">
                {why.item.provenance === "observed" ? `Observed — ${nameOf(house.members, why.item.corrected_by)} tapped it.` : "Inferred from receipts — computed in SQL, not by a model."}
              </p>
            </Section>
            <Section title={`Sources · ${why.receipts.length} receipt(s)`}>
              <ul className="space-y-3">
                {why.receipts.slice(0, 3).map((r) => (
                  <li key={r.receipt_id} className="rounded-xl border border-stone-200 p-3">
                    <div className="mb-1 flex flex-wrap items-center justify-between gap-2 text-xs text-stone-500">
                      <span>
                        {r.store} · {date(r.purchased_at)?.toLocaleDateString(undefined, { month: "short", day: "numeric" })} · {nameOf(house.members, r.member_id)}’s inbox
                      </span>
                      <span className="rounded bg-stone-100 px-1.5 py-0.5">{r.source === "airbyte" ? "synced by Airbyte" : r.source === "seeded" ? "seeded receipt" : r.source}</span>
                    </div>
                    <pre className="whitespace-pre-wrap font-mono text-xs leading-relaxed text-stone-700">
                      <Highlight text={r.raw_text} span={r.evidence_span} />
                    </pre>
                    <div className="mt-2 text-xs">
                      <span className={r.verified ? "font-semibold text-emerald-700" : "font-semibold text-red-700"}>{r.verified ? "✓ Line found verbatim in the receipt" : "✕ Line not found in the receipt — excluded"}</span>
                      <span className="ml-2 text-stone-500">extracted by {r.extractor}</span>
                    </div>
                  </li>
                ))}
              </ul>
            </Section>
            {why.options.length > 0 && (
              <Section title="Where to buy">
                <ul className="space-y-1 text-sm">
                  {why.options.map((o, i) => (
                    <li key={i} className="flex justify-between">
                      <span>
                        {o.product.channel === "online" ? "🚚" : "🛒"} {o.product.store} · {o.product.channel === "online" ? "delivery" : "pick up"}
                      </span>
                      <span className="font-semibold">{money(o.product.price_cents)}</span>
                    </li>
                  ))}
                </ul>
              </Section>
            )}
          </>
        )}

        {why?.plan && (
          <>
            <Section title="Decision">
              <div className="flex items-center gap-3">
                <StatusPill status={why.plan.status} />
                <span className="text-lg font-semibold">{money(why.plan.total_cents)}</span>
                <span className="text-xs text-stone-500">agent {why.plan.agent_version}</span>
              </div>
              {why.plan.decision_note && <p className="mt-2 text-sm">{why.plan.decision_note}</p>}
            </Section>
            <Section title="What was proposed">
              <ul className="space-y-2">
                {why.plan.basket.map((l) => (
                  <li key={l.item_id} className="text-sm">
                    <div className="flex justify-between">
                      <span className="font-medium">
                        {l.name} ×{l.qty}
                      </span>
                      <span>{l.product_id ? money(l.qty * l.unit_price_cents) : "no price"}</span>
                    </div>
                    <div className="text-xs text-stone-500">{l.sourcing_note}</div>
                    {l.why && <div className="text-xs text-stone-500">Because: {l.why}</div>}
                  </li>
                ))}
              </ul>
            </Section>
            <Section title="Checks · run in code, not by the model">
              <ul className="space-y-1.5">
                {why.plan.verdict.checks.map((c) => {
                  const s = CHECK_ICON[c.status];
                  return (
                    <li key={c.name} className="flex gap-2 text-sm">
                      <span className={`w-4 shrink-0 font-bold ${s.cls}`} aria-label={s.label}>
                        {s.icon}
                      </span>
                      <span>
                        {c.detail}
                        {c.status === "cannot_measure" && <span className="ml-1 font-semibold text-amber-700">(cannot measure)</span>}
                      </span>
                    </li>
                  );
                })}
              </ul>
            </Section>
          </>
        )}

        {why && why.events.length > 0 && (
          <Section title="Trail">
            <ol className="space-y-2 border-l-2 border-stone-200 pl-4">
              {why.events.map((e) => (
                <li key={e.id} className="text-sm">
                  <div>{e.summary}</div>
                  <div className="flex flex-wrap gap-x-3 text-xs text-stone-500">
                    <span>{timeLabel(e.ts)}</span>
                    <span>{e.actor === "agent" ? "Stocked" : nameOf(house.members, e.actor)}</span>
                    <span>{e.provenance}</span>
                    {e.model && <span className="font-mono">{e.model}</span>}
                    {e.latency_us > 0 && <span className="font-mono">{latency(e.latency_us)}</span>}
                    {e.cost_credits > 0 && <span className="font-mono">{e.cost_credits} credits</span>}
                  </div>
                </li>
              ))}
            </ol>
          </Section>
        )}
      </aside>
    </div>
  );
}
