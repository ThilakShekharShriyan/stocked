"use client";

// Typed client for the Rust core. One hook (`useHouse`) keeps the whole page live via SSE.

import { useCallback, useEffect, useRef, useState } from "react";

export const CORE = process.env.NEXT_PUBLIC_CORE_URL ?? "http://localhost:8080";

export type Member = { id: string; name: string; role: string };
export type Item = {
  id: string; name: string; category: string; unit: string; is_shared_consumable: boolean;
  purchases: number; interval_days: number | null; last_bought: string | null; runs_out_on: string | null;
  days_left: number | null; status: "low" | "soon" | "ok" | "cannot_estimate" | "event_only";
  basis: string; provenance: "observed" | "inferred"; correction: string | null; corrected_by: string | null;
};
export type Line = {
  item_id: string; name: string; qty: number; unit_price_cents: number; is_shared_consumable: boolean;
  product_id: string | null; store: string | null; channel: string | null; sourcing_note: string;
  arrives_in_time: boolean | null; why: string;
};
export type Check = { name: string; status: "pass" | "fail" | "cannot_measure"; detail: string };
export type Plan = {
  id: string; version: number; kind: string; title: string; trigger_ref: string | null; requested_by: string | null;
  basket: Line[]; verdict: { outcome: string; total_cents: number; checks: Check[]; reasons: string[] };
  total_cents: number; status: string; agent_version: string; created_at: string | null;
  decided_by: string | null; decided_at: string | null; decision_note: string | null;
};
export type Order = {
  id: string; plan_id: string; total_cents: number; paid_by: string; payment_provider: string; payment_ref: string;
  splits: { member_id: string; share_cents: number }[]; test_mode: boolean; created_at: string | null;
};
export type CalendarEvent = { id: string; title: string; starts_at: string | null; guests: number; kind: string; nights: number; host_member_id: string; source: string };
export type RideOption = { mode: "cab" | "transit"; label: string; total_cents: number; per_person_cents: number; extra_min: number };
export type Quote = { headcount: number; reservation_cents: number; ride: RideOption; alternatives: RideOption[]; total_cents: number; per_person_cents: number; over_cap: string[]; why: string };
export type Invitee = { member_id: string; name: string; status: "going" | "declined"; reason: string; by: string; cap_cents: number };
export type Outing = {
  id: string; version: number; title: string; kind: string; venue: string; starts_at: string | null; organizer: string;
  invitees: Invitee[]; quote: Quote; what_if: Quote[]; status: "planned" | "booked";
  pricing: { per_head_cents: number; backup?: { venue: string; per_head_cents: number } | null };
  booking: { reservation?: { ref: string; party_size: number }; ride?: { label: string }; disruption?: string } | null;
};
export type Verification = { id: string; member_id: string; plan_id: string | null; provider: "persona" | "persona_relay" | "simulated"; inquiry_id: string | null; status: string; url: string | null; valid_until: string | null };
export type AgentEvent = {
  id: string; ts: string | null; kind: string; subject_type: string; subject_id: string; actor: string; summary: string;
  data: Record<string, unknown>; model: string | null; latency_us: number; cost_credits: number; provenance: string;
};
export type House = {
  household: { id: string; name: string };
  budget: { monthly_cents: number; spent_cents: number; remaining_cents: number; auto_limit_cents: number; agent_version: string; config_source: string };
  payments: { provider: string; test_mode: boolean };
  world: { mode: string };
  database: { tables: { table: string; here: number; all: number }[]; catalog_products: number; households: number; event_kinds: { kind: string; n: number }[] };
  totals: { events: number; events_forwarded: number; model_steps: number; model_credits: number; receipts: number; receipts_synced: number; amplitude_configured: boolean; statsig_connected: boolean };
  members: Member[]; items: Item[]; calendar: CalendarEvent[]; plans: Plan[]; orders: Order[];
  balances: { member_id: string; name: string; net_cents: number }[];
  settlements: { from: string; to: string; cents: number }[];
  recent_events: AgentEvent[];
  verifications: Verification[];
  outings: Outing[];
  identity: { provider: "persona" | "persona_relay" | "simulated"; sandbox: boolean; valid_minutes: number };
};
export type Why = {
  subject_id: string; item: Item | null; plan: Plan | null; events: AgentEvent[];
  receipts: { receipt_id: string; source: string; source_ref: string | null; store: string; subject: string; raw_text: string; member_id: string; qty: number; unit_price_cents: number; purchased_at: string | null; evidence_span: string; verified: boolean; extractor: string }[];
  options: { product: { store: string; channel: string; title: string; price_cents: number }; order_by: string | null; arrives_at: string; feasible: boolean }[];
};

export async function api<T = unknown>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${CORE}${path}`, {
    method: body === undefined ? "GET" : "POST",
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    cache: "no-store",
  });
  const json = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error((json as { error?: string }).error ?? `HTTP ${res.status}`);
  return json as T;
}

export function useHouse() {
  const [house, setHouse] = useState<House | null>(null);
  const [error, setError] = useState<string | null>(null);
  const busy = useRef(false);

  const refresh = useCallback(async () => {
    if (busy.current) return;
    busy.current = true;
    try {
      setHouse(await api<House>("/api/state"));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      busy.current = false;
    }
  }, []);

  useEffect(() => {
    refresh();
    // ?static=1 loads once without the live connection (used for screenshots).
    if (new URLSearchParams(window.location.search).get("static")) return;
    const es = new EventSource(`${CORE}/api/stream`);
    es.addEventListener("change", refresh);
    const poll = setInterval(refresh, 8000); // safety net if the stream drops
    return () => {
      es.close();
      clearInterval(poll);
    };
  }, [refresh]);

  return { house, error, refresh };
}

// ---- formatting ---------------------------------------------------------------------------

export const money = (cents: number) => `$${(cents / 100).toFixed(2)}`;

/** Server timestamps are naive UTC. */
export const date = (s: string | null) => (s ? new Date(s.endsWith("Z") ? s : `${s}Z`) : null);

export function dayLabel(s: string | null): string {
  const d = date(s);
  if (!d) return "—";
  const days = Math.round((d.getTime() - Date.now()) / 86_400_000);
  if (days <= 0) return "today";
  if (days === 1) return "tomorrow";
  return d.toLocaleDateString(undefined, { weekday: "short" }) + (days > 6 ? ` ${d.toLocaleDateString(undefined, { month: "short", day: "numeric" })}` : "");
}

export function timeLabel(s: string | null): string {
  const d = date(s);
  return d ? d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" }) : "";
}

export function latency(us: number): string {
  if (us <= 0) return "";
  if (us < 1000) return `${us} µs`;
  if (us < 1_000_000) return `${(us / 1000).toFixed(us < 10_000 ? 1 : 0)} ms`;
  return `${(us / 1_000_000).toFixed(1)} s`;
}

export const nameOf = (members: Member[], id: string | null) => members.find((m) => m.id === id)?.name ?? (id === "agent" ? "Stocked" : id ?? "");

/** Items already covered by a recent order or a plan awaiting approval: item id -> label. */
export function covered(house: House): Map<string, string> {
  const m = new Map<string, string>();
  for (const p of house.plans) {
    const label = p.status === "ordered" ? "On order" : p.status === "held" ? "In a plan awaiting approval" : null;
    if (!label) continue;
    for (const l of p.basket) if (!m.has(l.item_id) || label === "On order") m.set(l.item_id, label);
  }
  return m;
}

/** The member's unexpired identity verification, if any. */
export function verified(house: House, memberId: string): Verification | undefined {
  return house.verifications.find((v) => v.member_id === memberId && v.status === "approved" && (date(v.valid_until)?.getTime() ?? 0) > Date.now());
}

export function eventSummary(e: CalendarEvent): string {
  if (e.kind === "houseguests") return `${e.guests} guest${e.guests === 1 ? "" : "s"} staying ${e.nights} night${e.nights === 1 ? "" : "s"}`;
  if (e.kind === "away") return `away ${e.nights} night${e.nights === 1 ? "" : "s"}`;
  if (e.kind === "dinner") return `dinner for ${e.guests}`;
  return `${e.guests} guests`;
}
