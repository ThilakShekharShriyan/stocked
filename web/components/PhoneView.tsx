"use client";

// What one flatmate sees on their phone: only cards that need a tap.
// Used inside the dashboard's phone frame and on its own at /phone/[person].

import { useEffect, useRef, useState } from "react";
import { House, Verification, api, covered, money, nameOf, verified } from "@/lib/api";

function Card({ children, tone = "plain" }: { children: React.ReactNode; tone?: "plain" | "ask" | "warn" }) {
  const cls = tone === "ask" ? "border-amber-300 bg-amber-50" : tone === "warn" ? "border-red-300 bg-red-50" : "border-stone-200 bg-white";
  return <div className={`rounded-2xl border p-4 ${cls}`}>{children}</div>;
}

function Tap({ onClick, primary, children }: { onClick: () => Promise<unknown>; primary?: boolean; children: React.ReactNode }) {
  const [busy, setBusy] = useState(false);
  return (
    <button
      disabled={busy}
      onClick={async () => {
        setBusy(true);
        try {
          await onClick();
        } finally {
          setBusy(false);
        }
      }}
      className={`min-h-12 flex-1 rounded-xl px-4 text-base font-semibold disabled:opacity-50 ${primary ? "bg-teal-800 text-white hover:bg-teal-900" : "border border-stone-300 bg-white hover:bg-stone-50"}`}
    >
      {busy ? "…" : children}
    </button>
  );
}

/** Persona Relay: the widget runs the live-human check; our server redeems the claim and tells the core. */
function RelayCheck({ house, memberId, planId, onChanged, onError }: { house: House; memberId: string; planId: string; onChanged: () => void; onError: (m: string) => void }) {
  const box = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<"idle" | "starting" | "open" | "checking">("idle");
  const relay = useRef<{ destroy: () => void } | null>(null);
  useEffect(() => () => relay.current?.destroy(), []);

  const start = async () => {
    setState("starting");
    try {
      const res = await fetch("/api/relay/session", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ member: memberId, plan_id: planId }) });
      const body = await res.json();
      if (!res.ok) throw new Error(body.error ?? "Could not start the check");
      const { default: Relay } = await import("@persona/relay");
      setState("open");
      relay.current = new Relay(box.current!, {
        accessToken: body.access_token,
        theme: "light",
        onComplete: async () => {
          setState("checking");
          const r = await fetch("/api/relay/redeem", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ verification_id: body.verification_id }) });
          const out = await r.json();
          if (!r.ok) onError(out.error ?? "The check could not be confirmed");
          relay.current?.destroy();
          setState("idle");
          onChanged();
        },
        onExpire: () => {
          onError("The check expired. Start it again.");
          setState("idle");
        },
        onError: (e: unknown) => {
          onError(e instanceof Error ? e.message : "The check failed to load");
          setState("idle");
        },
      });
    } catch (e) {
      onError(e instanceof Error ? e.message : String(e));
      setState("idle");
    }
  };

  return (
    <div className="mt-3">
      {state === "idle" && (
        <>
          <p className="mb-2 text-xs text-stone-600">Over the auto-approve limit, so Persona checks that a real person is approving this.</p>
          <div className="flex gap-2">
            <Tap primary onClick={start}>
              Verify it’s you
            </Tap>
          </div>
          <p className="mt-2 text-xs text-stone-500">Persona Relay{house.identity.sandbox ? " · sandbox" : ""}. No identity data is shared with Stocked, only pass or fail.</p>
        </>
      )}
      {state === "starting" && <p className="text-sm text-stone-600">Starting the check…</p>}
      {state === "checking" && <p className="text-sm text-stone-600">Confirming with Persona…</p>}
      <div ref={box} className={state === "open" ? "mt-2 min-h-64 overflow-hidden rounded-xl border border-stone-200 bg-white" : "hidden"} />
    </div>
  );
}

/** Step-up identity check before an over-limit approval: Persona's hosted flow, or a labelled simulation. */
function VerifyStep({ house, memberId, planId, pending, onChanged, onError }: { house: House; memberId: string; planId: string; pending?: Verification; onChanged: () => void; onError: (m: string) => void }) {
  const run = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
    } catch (e) {
      onError(e instanceof Error ? e.message : String(e));
    }
    onChanged();
  };
  if (house.identity.provider === "persona_relay") return <RelayCheck house={house} memberId={memberId} planId={planId} onChanged={onChanged} onError={onError} />;
  const persona = house.identity.provider === "persona";
  if (!pending) {
    return (
      <div className="mt-3">
        <p className="mb-2 text-xs text-stone-600">Over the auto-approve limit, so I need to know it’s really you.</p>
        <div className="flex gap-2">
          <Tap
            primary
            onClick={() =>
              run(async () => {
                const r = await api<{ verification: Verification }>("/api/verify/start", { member: memberId, plan_id: planId });
                if (r.verification.url) window.open(r.verification.url, "_blank", "noopener");
              })
            }
          >
            Verify it’s you
          </Tap>
        </div>
        {!persona && <p className="mt-2 text-xs text-stone-500">Persona isn’t connected, so this check will be simulated.</p>}
      </div>
    );
  }
  return (
    <div className="mt-3">
      {persona ? (
        <>
          <p className="mb-2 text-xs text-stone-600">
            Finish the Persona check{house.identity.sandbox ? " (sandbox)" : ""}, then come back.{" "}
            {pending.url && (
              <a className="font-semibold text-teal-800 underline" href={pending.url} target="_blank" rel="noopener noreferrer">
                Open it again
              </a>
            )}
          </p>
          <div className="flex gap-2">
            <Tap primary onClick={() => run(() => api(`/api/verify/${pending.id}/check`, {}))}>
              I’ve finished
            </Tap>
          </div>
        </>
      ) : (
        <>
          <p className="mb-2 text-xs font-semibold text-violet-900">SIMULATED CHECK · Persona not connected</p>
          <div className="flex gap-2">
            <Tap primary onClick={() => run(() => api(`/api/verify/${pending.id}/simulate`, {}))}>
              Simulate a passed check
            </Tap>
          </div>
        </>
      )}
    </div>
  );
}

export function PhoneView({ house, person, onChanged }: { house: House; person: string; onChanged: () => void }) {
  const me = house.members.find((m) => m.id === person);
  const [ask, setAsk] = useState("");
  const [note, setNote] = useState<string | null>(null);
  if (!me) return <p className="p-4 text-sm">Unknown flatmate.</p>;

  const act = async (path: string, body: object) => {
    setNote(null);
    try {
      await api(path, body);
    } catch (e) {
      setNote(e instanceof Error ? e.message : String(e));
    }
    onChanged();
  };

  const isApprover = me.role === "approver";
  const proof = verified(house, me.id);
  const pending = house.verifications.find((v) => v.member_id === me.id && v.status === "pending");
  const held = house.plans.filter((p) => p.status === "held");
  const failed = house.plans.filter((p) => p.status === "payment_failed");
  const share = (total: number) => money(Math.round(total / Math.max(house.members.length, 1)));
  const cov = covered(house);
  const asks = house.items.filter((i) => (i.status === "low" || i.status === "soon") && i.provenance === "inferred" && !cov.has(i.id)).slice(0, 2);
  const net = house.balances.find((b) => b.member_id === me.id)?.net_cents ?? 0;
  const mine = house.plans.filter((p) => p.requested_by === me.id && p.kind === "request").slice(0, 2);

  return (
    <div className="space-y-3 p-3">
      <div className="px-1">
        <div className="text-xs text-stone-500">{house.household.name}</div>
        <div className="text-xl font-semibold">Hi {me.name}</div>
      </div>

      {note && (
        <Card tone="warn">
          <p className="text-sm">{note}</p>
        </Card>
      )}

      {held.map((p) => (
        <Card key={p.id} tone="ask">
          <div className="text-xs font-semibold uppercase tracking-wide text-amber-800">Needs approval</div>
          <div className="mt-1 text-base font-semibold">{p.title}</div>
          <div className="text-sm text-stone-700">
            {money(p.total_cents)} · {p.basket.length} items · {share(p.total_cents)} each
          </div>
          <div className="mt-1 text-xs text-stone-600">{p.verdict.reasons[0]}</div>
          {isApprover && !proof ? (
            <VerifyStep house={house} memberId={me.id} planId={p.id} pending={pending} onChanged={onChanged} onError={setNote} />
          ) : isApprover ? (
            <>
              <p className="mt-2 text-xs font-semibold text-emerald-800">
                ✓ {proof?.provider === "persona_relay" ? "Live human confirmed by Persona Relay" : proof?.provider === "simulated" ? "Identity verified (simulated)" : "Identity verified by Persona"}
              </p>
              <div className="mt-2 flex gap-2">
                <Tap primary onClick={() => act(`/api/plans/${p.id}/approve`, { member: me.id })}>
                  Approve
                </Tap>
                <Tap onClick={() => act(`/api/plans/${p.id}/skip`, { member: me.id, reason: "Skipped from phone" })}>Skip</Tap>
              </div>
            </>
          ) : (
            <p className="mt-2 text-sm text-stone-600">Waiting for {nameOf(house.members, house.members.find((m) => m.role === "approver")?.id ?? null)}.</p>
          )}
        </Card>
      ))}

      {isApprover &&
        failed.map((p) => (
          <Card key={p.id} tone="warn">
            <div className="text-base font-semibold">Payment didn’t go through</div>
            <div className="text-sm">{p.title} · nothing was charged.</div>
            {proof ? (
              <div className="mt-3 flex gap-2">
                <Tap primary onClick={() => act(`/api/plans/${p.id}/approve`, { member: me.id })}>
                  Try again
                </Tap>
              </div>
            ) : (
              <VerifyStep house={house} memberId={me.id} planId={p.id} pending={pending} onChanged={onChanged} onError={setNote} />
            )}
          </Card>
        ))}

      {house.outings.map((o) => {
        const mine = o.invitees.find((i) => i.member_id === me.id);
        if (!mine) return null;
        const organiser = o.organizer === me.id;
        const needsCheck = o.quote.total_cents > house.budget.auto_limit_cents && !proof;
        return (
          <Card key={o.id} tone={organiser && o.status !== "booked" ? "ask" : "plain"}>
            <div className="text-xs font-semibold uppercase tracking-wide text-stone-500">{o.status === "booked" ? `Booked for ${o.quote.headcount}` : `Planned for ${o.quote.headcount}`}</div>
            <div className="mt-1 text-base font-semibold">{o.title}</div>
            <div className="text-sm text-stone-700">
              {money(o.quote.per_person_cents)} each · {o.quote.ride.label.toLowerCase()}
            </div>
            <div className="mt-1 text-xs text-stone-600">
              {mine.status === "going" ? "You’re in" : "You’re out"} — {mine.reason}
            </div>
            {organiser && o.status !== "booked" && needsCheck && <VerifyStep house={house} memberId={me.id} planId={o.id} pending={pending} onChanged={onChanged} onError={setNote} />}
            <div className="mt-3 flex gap-2">
              {organiser && o.status !== "booked" && !needsCheck && (
                <Tap primary onClick={() => act(`/api/outings/${o.id}/book`, { member: me.id })}>
                  Book for {o.quote.headcount}
                </Tap>
              )}
              {mine.status === "going" ? (
                <Tap onClick={() => act(`/api/outings/${o.id}/rsvp`, { member: me.id, status: "declined" })}>Can’t make it</Tap>
              ) : (
                <Tap onClick={() => act(`/api/outings/${o.id}/rsvp`, { member: me.id, status: "going" })}>I’m in</Tap>
              )}
            </div>
          </Card>
        );
      })}

      {asks.map((i) => (
        <Card key={i.id}>
          <div className="text-base font-semibold">{i.name}: still have some?</div>
          <div className="text-xs text-stone-500">I think it runs out soon — {i.basis.toLowerCase()}.</div>
          <div className="mt-3 flex gap-2">
            <Tap onClick={() => act(`/api/items/${i.id}/correct`, { member: me.id, level: "out" })}>We’re out</Tap>
            <Tap onClick={() => act(`/api/items/${i.id}/correct`, { member: me.id, level: "plenty" })}>Plenty left</Tap>
          </div>
        </Card>
      ))}

      <Card>
        <div className="text-sm text-stone-600">This month</div>
        <div className="text-base font-semibold">{net === 0 ? "You’re square" : net > 0 ? `You’re owed ${money(net)}` : `You owe ${money(-net)}`}</div>
      </Card>

      {mine.map((p) => (
        <Card key={p.id} tone={p.status === "refused" ? "warn" : "plain"}>
          <div className="text-sm font-semibold">{p.title}</div>
          <div className="text-sm">{p.status === "refused" ? `Not bought: ${p.decision_note}` : p.status === "ordered" ? `Ordered · ${money(p.total_cents)}` : "Waiting for approval"}</div>
        </Card>
      ))}

      <Card>
        <label htmlFor={`ask-${me.id}`} className="text-sm font-semibold">
          Ask the house for something
        </label>
        <div className="mt-2 flex gap-2">
          <input
            id={`ask-${me.id}`}
            value={ask}
            onChange={(e) => setAsk(e.target.value)}
            placeholder="e.g. bluetooth speaker"
            className="min-h-12 w-full min-w-0 flex-1 rounded-xl border border-stone-300 px-3 text-base"
          />
          <button
            disabled={!ask.trim()}
            onClick={async () => {
              const text = ask.trim();
              setAsk("");
              await act("/api/request", { member: me.id, text });
            }}
            className="min-h-12 rounded-xl bg-stone-900 px-4 text-base font-semibold text-white disabled:opacity-40"
          >
            Ask
          </button>
        </div>
      </Card>
    </div>
  );
}
