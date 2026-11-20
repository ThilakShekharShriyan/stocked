"use client";

import dynamic from "next/dynamic";
import { useCallback, useEffect, useMemo, useState } from "react";
import { api, money, useHouse } from "@/lib/api";
import { EDGES, Tone, buildNodes, routeOf, toneOf } from "@/lib/world";
import type { Pulse } from "@/components/AgentWorld";
import { ActivityStream, AutonomySummary, Badge, Card, NodeDetails, NowPanel } from "@/components/overview";
import { AuditTrail, ScenarioLab, SuppliesFlow } from "@/components/sections";
import { ComingUp, Ledger, RunningLow } from "@/components/panels";
import { Outings } from "@/components/Outings";
import { PhoneView } from "@/components/PhoneView";
import { WhyDrawer } from "@/components/WhyDrawer";
import { InsightsTab } from "@/components/tabs";

// The scene loads after the page, so it never blocks first paint.
const AgentWorld = dynamic(() => import("@/components/AgentWorld").then((m) => m.AgentWorld), {
  ssr: false,
  loading: () => <div className="grid h-full place-items-center rounded-2xl border border-line bg-surface text-sm text-stone-500">Loading the agent world…</div>,
});

const SECTIONS = ["Overview", "Supplies", "Outings", "Scenario lab", "Audit trail", "Insights"] as const;
type Section = (typeof SECTIONS)[number];

export default function Stocked() {
  const { house, error, refresh } = useHouse();
  const [section, setSection] = useState<Section>("Overview");
  const [why, setWhy] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [phone, setPhone] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [whatIf, setWhatIf] = useState<{ id: string; n: number } | null>(null);
  const closeWhy = useCallback(() => setWhy(null), []);
  // Deep links for demos and screenshots: /?section=Outings&phone=maya&select=persona
  useEffect(() => {
    const q = new URLSearchParams(window.location.search);
    const sec = q.get("section");
    if (sec && (SECTIONS as readonly string[]).includes(sec)) setSection(sec as Section);
    if (q.get("phone")) setPhone(q.get("phone"));
    if (q.get("select")) setSelected(q.get("select"));
    if (q.get("why")) setWhy(q.get("why"));
  }, []);

  const nodes = useMemo(() => (house ? buildNodes(house) : []), [house]);
  const events = useMemo(() => (house ? house.recent_events.filter((e) => e.kind !== "turn_score") : []), [house]);

  const scene = useMemo(() => {
    if (!house) return { pulses: [] as Pulse[], active: [] as string[], current: null as string | null, tones: {} as Record<string, Tone> };
    const routed = events.map((e) => ({ e, r: routeOf(e, house) })).filter((x): x is { e: (typeof events)[number]; r: { from: string; to: string } } => !!x.r);
    const pulses: Pulse[] = routed.slice(0, 10).map(({ e, r }) => ({ ...r, tone: toneOf(e.kind) }));
    const active = Array.from(new Set(routed.slice(0, 6).flatMap(({ r }) => [r.from, r.to])));
    const tones: Record<string, Tone> = {};
    if (house.plans.some((p) => p.status === "held" || p.status === "payment_failed")) tones.persona = "ember";
    if (house.orders.length > 0) tones.payments = "sage";
    return { pulses, active, current: routed[0]?.r.to ?? null, tones };
  }, [house, events]);

  if (!house) {
    return (
      <main className="grid min-h-screen place-items-center p-8 text-center">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Stocked</h1>
          <p className="mt-2 text-stone-500">{error ? `Can’t reach the agent right now (${error}).` : "Waking the household…"}</p>
        </div>
      </main>
    );
  }

  const run = async (label: string, path: string) => {
    setBusy(label);
    try { await api(path, {}); } catch { /* surfaced through state */ }
    setBusy(null);
    refresh();
  };
  const waiting = house.plans.filter((p) => p.status === "held" || p.status === "payment_failed").length;
  const selectedNode = nodes.find((n) => n.id === selected) ?? null;

  // Outings: the slider decides who is shown as taking part.
  const outing = house.outings.find((o) => o.id === whatIf?.id) ?? house.outings[0];
  const n = whatIf && outing && whatIf.id === outing.id ? whatIf.n : outing?.quote.headcount ?? 0;
  const order = outing ? [...outing.invitees.filter((i) => i.status === "going"), ...outing.invitees.filter((i) => i.status !== "going")] : [];
  const quote = outing ? outing.what_if[n - 1] ?? outing.quote : null;
  const outingScene = {
    emphasis: ["hub", "outing", "calendars", "payments", "persona", "cab", "transit", ...house.members.map((m) => `p:${m.id}`)],
    absent: order.slice(n).map((i) => `p:${i.member_id}`),
    tones: { ...(quote ? { [quote.ride.mode]: "sage" as Tone } : {}), ...Object.fromEntries((quote?.over_cap ?? []).filter((id) => !id.startsWith("guest")).map((id) => [`p:${id}`, "clay" as Tone])) },
    pulses: scene.pulses.filter((p) => p.from === "outing" || p.to === "outing"),
  };

  return (
    <div className="flex min-h-screen flex-col lg:flex-row">
      {/* Navigation rail */}
      <aside className="flex shrink-0 flex-col gap-4 border-b border-line bg-surface p-4 lg:sticky lg:top-0 lg:h-screen lg:w-60 lg:overflow-y-auto lg:border-b-0 lg:border-r">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Stocked</h1>
          <p className="text-xs leading-snug text-stone-500">The shopping agent for people who share a home</p>
        </div>
        <div className="flex flex-wrap gap-1.5">
          <Badge tone="violet">TEST MODE</Badge>
          <Badge tone="violet">{house.identity.provider === "simulated" ? "IDENTITY SIMULATED" : "PERSONA SANDBOX"}</Badge>
          <Badge tone="sage">● Active</Badge>
        </div>
        <nav aria-label="Sections" className="flex gap-1 overflow-x-auto lg:flex-col lg:overflow-visible">
          {SECTIONS.map((s) => (
            <button key={s} onClick={() => setSection(s)} aria-current={section === s ? "page" : undefined} className={`whitespace-nowrap rounded-lg px-3 py-2 text-left text-sm font-medium ${section === s ? "bg-raised text-ivory" : "text-stone-500 hover:bg-raised hover:text-ivory"}`}>
              {s}
              {s === "Overview" && waiting > 0 && <span className="ml-2 rounded-full bg-ember px-1.5 text-[11px] font-bold text-ink">{waiting}</span>}
            </button>
          ))}
        </nav>
        <div className="rounded-xl border border-line p-3">
          <div className="text-[11px] uppercase tracking-widest text-stone-500">{house.household.name}</div>
          <div className="mt-1 text-lg font-semibold tabular-nums">{money(house.budget.remaining_cents)} <span className="text-xs font-normal text-stone-500">of {money(house.budget.monthly_cents)} left</span></div>
          <div className="mt-2 h-1.5 rounded-full bg-raised"><div className="h-1.5 rounded-full bg-sage" style={{ width: `${Math.max(2, (house.budget.remaining_cents / house.budget.monthly_cents) * 100)}%` }} /></div>
          <div className="mt-3 text-[11px] uppercase tracking-widest text-stone-500">Phones</div>
          <div className="mt-1 flex flex-wrap gap-1">
            {house.members.map((m) => (
              <button key={m.id} onClick={() => setPhone(m.id)} className="rounded-full border border-line px-2.5 py-1 text-xs font-medium hover:bg-raised">
                {m.name}{m.role === "approver" && waiting > 0 ? " ●" : ""}
              </button>
            ))}
          </div>
        </div>
        <div className="mt-auto space-y-1.5">
          <div className="text-[11px] uppercase tracking-widest text-stone-500">Demo controls</div>
          {([["sync", "Sync inbox and calendars", "/api/airbyte/sync", "Syncing… (about 35 s)"], ["invite", "Simulate a calendar invite", "/api/invite/simulate", "Planning…"], ["scan", "Run restock scan", "/api/scan", "Scanning…"], ["reset", "Reset the flat", "/api/reset", "Resetting…"]] as const).map(([k, label, path, wait]) => (
            <button key={k} disabled={!!busy} onClick={() => run(k, path)} className="block w-full rounded-lg border border-line px-3 py-1.5 text-left text-xs font-medium hover:bg-raised disabled:opacity-50">
              {busy === k ? wait : label}
            </button>
          ))}
        </div>
      </aside>

      <main className="min-w-0 flex-1 p-4 lg:p-5">
        {section === "Overview" && (
          <p className="mb-4 max-w-4xl text-2xl font-semibold leading-snug tracking-tight sm:text-[28px]">
            Watching {house.calendar.length} calendar event{house.calendar.length === 1 ? "" : "s"} and {house.items.filter((i) => i.runs_out_on).length} supplies for {house.members.length} people.{" "}
            {waiting > 0 ? (
              <span className="text-ember">{waiting} decision{waiting === 1 ? "" : "s"} need{waiting === 1 ? "s" : ""} {house.members.find((m) => m.role === "approver")?.name ?? "a person"}.</span>
            ) : (
              <span className="text-sage">Nothing needs anyone right now.</span>
            )}
          </p>
        )}
        {section === "Overview" && (
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
            <div className="min-w-0 space-y-4">
              <div className="h-[440px] sm:h-[min(64vh,640px)]">
                <AgentWorld nodes={nodes} edges={EDGES} pulses={scene.pulses} selected={selected} onSelect={setSelected} tones={scene.tones} active={scene.active} currentId={scene.current} className="h-full" />
              </div>
              <Card title="Agent activity" hint="who told whom what · newest first">
                <ActivityStream house={house} nodes={nodes} events={events} onWhy={setWhy} onSelectNode={setSelected} />
              </Card>
            </div>
            <div className="space-y-4">
              <NowPanel house={house} onWhy={setWhy} onOpenPhone={setPhone} />
              {selectedNode ? <NodeDetails node={selectedNode} house={house} events={events} onWhy={setWhy} onClose={() => setSelected(null)} /> : null}
              <AutonomySummary house={house} />
            </div>
          </div>
        )}

        {section === "Supplies" && (
          <div className="space-y-4">
            <SuppliesFlow house={house} onWhy={setWhy} onStage={setSelected} />
            <div className="grid gap-4 lg:grid-cols-3">
              <RunningLow house={house} onWhy={setWhy} />
              <ComingUp house={house} onWhy={setWhy} />
              <Ledger house={house} onWhy={setWhy} />
            </div>
          </div>
        )}

        {section === "Outings" && (
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <div className="space-y-3">
              <div className="h-[380px] sm:h-[460px] xl:sticky xl:top-5">
                <AgentWorld nodes={nodes} edges={EDGES} pulses={outingScene.pulses} selected={selected} onSelect={setSelected} tones={outingScene.tones} absent={outingScene.absent} emphasis={outingScene.emphasis} active={["outing"]} currentId="outing" className="h-full" />
              </div>
              {outing && quote && (
                <p className="px-1 text-sm text-stone-600" aria-live="polite">
                  <b className="text-ivory">{outing.title}</b> with {n} {n === 1 ? "person" : "people"}: {quote.ride.label.toLowerCase()}, {money(quote.per_person_cents)} each.
                  {n > order.length ? ` Includes ${n - order.length} guest${n - order.length === 1 ? "" : "s"} from outside the flat.` : ""}
                  {quote.over_cap.filter((id) => !id.startsWith("guest")).length > 0 ? " Someone is over their limit (shown in red)." : ""}
                </p>
              )}
              {selectedNode && <NodeDetails node={selectedNode} house={house} events={events} onWhy={setWhy} onClose={() => setSelected(null)} />}
            </div>
            <Outings house={house} onWhy={setWhy} onWhatIf={(id, k) => setWhatIf({ id, n: k })} activeId={outing?.id ?? null} />
            {house.outings.length === 0 && <Card><p className="text-sm text-stone-500">No outings yet. Press “Sync inbox and calendars”.</p></Card>}
          </div>
        )}

        {section === "Scenario lab" && <ScenarioLab />}
        {section === "Audit trail" && <AuditTrail house={house} nodes={nodes} onWhy={setWhy} />}
        {section === "Insights" && <InsightsTab house={house} />}
      </main>

      {/* Phone slide-over: the only place people act. */}
      {phone && (
        <div className="fixed inset-0 z-30 flex justify-end bg-black/50" onClick={() => setPhone(null)}>
          <aside aria-label="Phone view" className="flex h-full w-full max-w-sm flex-col border-l border-line bg-ink p-4" onClick={(e) => e.stopPropagation()}>
            <div className="mb-3 flex items-center justify-between gap-2">
              <div className="flex gap-1 rounded-lg border border-line p-1" role="group" aria-label="Whose phone">
                {house.members.map((m) => (
                  <button key={m.id} onClick={() => setPhone(m.id)} aria-pressed={phone === m.id} className={`rounded-md px-2.5 py-1 text-sm font-medium ${phone === m.id ? "bg-raised text-ivory" : "text-stone-500"}`}>{m.name}</button>
                ))}
              </div>
              <button onClick={() => setPhone(null)} className="rounded-lg border border-line px-2.5 py-1 text-xs font-medium hover:bg-raised">Close</button>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto rounded-[1.6rem] border-4 border-stone-300 bg-raised">
              <PhoneView house={house} person={phone} onChanged={refresh} />
            </div>
          </aside>
        </div>
      )}

      <WhyDrawer subject={why} house={house} onClose={closeWhy} />
    </div>
  );
}
