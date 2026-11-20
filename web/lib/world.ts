// The agent world: who and what exists in Stocked, where it sits in the scene, and which real
// events travel between which nodes. Nothing here invents activity; it only maps recorded events.

import type { AgentEvent, House } from "./api";

export type NodeKind = "hub" | "person" | "agent" | "tool" | "checkpoint" | "data";
export type Tone = "ivory" | "sage" | "ember" | "clay" | "mist";

export type WorldNode = {
  id: string;
  label: string;
  kind: NodeKind;
  role: string;
  /** How this node really works, in one or two plain sentences. */
  detail: string;
  /** Sponsor or mechanism behind it, if any. */
  poweredBy?: string;
  angle: number; // degrees around the hub
  radius: number;
  height: number;
};

export type WorldEdge = { from: string; to: string };

const A = (id: string, label: string, role: string, detail: string, poweredBy: string, angle: number): WorldNode => ({ id, label, kind: "agent", role, detail, poweredBy, angle, radius: 6.4, height: 0.5 });
const T = (id: string, label: string, role: string, detail: string, angle: number, radius = 9.6): WorldNode => ({ id, label, kind: "tool", role, detail, poweredBy: label, angle, radius, height: 0.25 });

const STATIC: WorldNode[] = [
  { id: "hub", label: "House agent", kind: "hub", role: "Coordinates every decision", detail: "The Rust core. It routes each request, runs every rule in code, and is the only thing that can place a test-mode order.", poweredBy: "Rust · axum", angle: 0, radius: 0, height: 0.9 },
  A("reader", "Receipt reader", "Reads receipt lines", "A small extraction model pulls item, quantity and price from each receipt. A line only counts if it appears word-for-word in the receipt.", "Superlinked SIE · gliner2", 205),
  A("estimator", "Run-out estimator", "Works out when things run out", "No model. SQL over the purchase history: how often the flat re-buys an item gives its run-out date. One-tap corrections override it.", "SingleStore SQL", 165),
  A("planner", "Basket planner", "Proposes what to buy", "Proposes a basket for an event or a low item. A Statsig gate chooses between a per-guest rule and an open LLM. It only proposes; it cannot order.", "Statsig gate · Superlinked SIE", 122),
  A("rules", "Rule check", "Decides if a plan may proceed", "Code, not a model: prices from the catalog, the monthly budget, the auto-approve limit, shared supplies only, and no double-buying.", "Rust checks", 78),
  A("classifier", "Item classifier", "Sorts requests into categories", "A decision model classifies a requested item (cleaning supply, food, electronics…). Non-supplies are refused.", "Superlinked SIE · GLiNER2.5-Decide", 38),
  A("outing", "Outing planner", "Prices plans by headcount", "Plain arithmetic: reservation per head plus a ride split by headcount. It picks the ride that keeps everyone within their own limit.", "Rust", 322),
  A("audit", "Audit log", "Records every step", "Every step of every agent is appended here with its inputs, model, time and cost. The Why panels and this scene are drawn from it.", "SingleStore", 262),
  { id: "persona", label: "Human check", kind: "checkpoint", role: "A person, not an agent", detail: "Before an over-limit approval or booking, Persona confirms a live human is present. Stocked never completes this step itself.", poweredBy: "Persona Relay · sandbox", angle: 58, radius: 4.9, height: 1.5 },
  T("airbyte", "Airbyte", "Syncs the inbox and calendars", "Open-source connector (PyAirbyte) syncing two streams: the flat's receipt inbox and every flatmate's calendar. The data is fictional files.", 220),
  T("sie", "Superlinked SIE", "Runs the models", "Serves three open models: one drives the agent, one reads receipts, one classifies items.", 100),
  T("singlestore", "SingleStore", "The only database", "Purchases, plans, orders, outings, verifications and the append-only event log all live here.", 248),
  T("statsig", "Statsig", "Gates and house rules", "A feature gate switches the planner between rules and the model. A dynamic config holds the auto-approve limit.", 140),
  T("amplitude", "Amplitude", "Agent analytics", "Every conversation is sent as a session: user message, tool calls, reply and code-side scores.", 285),
  T("firedrill", "Firedrill", "Simulated flatmates", "Simulated flatmates drive the agent against a synthetic Stripe. Exact checks count the charges.", 350),
  T("webworld", "Fictional web", "Venue pages, a cab site, an airline", "The agent reads venue pages and submits booking forms. In a drill this is Firedrill's synthetic web; in the dashboard it is a built-in stand-in. Every site is fictional.", 322, 10.6),
  T("gcal", "Calendars (synthetic)", "Free and busy times", "In a drill, a synthetic Google Calendar holds each flatmate's calendar and the agent asks it who is free.", 196, 10.4),
  T("mailbox", "Mail (synthetic)", "Confirmation emails", "The agent emails the plan to the people going. In a drill this is a synthetic Gmail; no real email is sent.", 268, 10.6),
  T("payments", "Test payments", "Test-mode charges only", "Every order and booking is a test-mode charge. No real money moves and nothing real is reserved.", 15, 8.2),
  { id: "cab", label: "Cab", kind: "data", role: "Fixed fare, split by headcount", detail: "A standard car seats four; five or six need a larger one. The fare is split between the people going. Simulated fares.", angle: 304, radius: 8.6, height: 0.2 },
  { id: "transit", label: "Transit", kind: "data", role: "A fare per person", detail: "Costs the same per person whatever the headcount, but takes longer. Simulated fares.", angle: 338, radius: 8.9, height: 0.2 },
  { id: "inbox", label: "Receipt inbox", kind: "data", role: "Where receipts arrive", detail: "Receipt emails for the flat. In this demo they are fictional and read from a file.", angle: 232, radius: 8.1, height: 0.2 },
  { id: "calendars", label: "Calendars", kind: "data", role: "One per flatmate", detail: "Each person's calendar: parties, guests staying, trips away, outings.", angle: 188, radius: 8.6, height: 0.2 },
];

const PERSON_ANGLES = [58, 148, 238, 328, 13, 103];

export function buildNodes(house: House): WorldNode[] {
  const people: WorldNode[] = house.members.map((m, i) => ({
    id: `p:${m.id}`,
    label: m.name,
    kind: "person",
    role: m.role === "approver" ? "Flatmate · approver" : "Flatmate",
    detail: `${m.name}'s own agent answers invitations from their calendar and spending limit.`,
    angle: PERSON_ANGLES[i % PERSON_ANGLES.length],
    radius: 3.3,
    height: 0.55,
  }));
  return [...STATIC, ...people];
}

export const EDGES: WorldEdge[] = [
  { from: "airbyte", to: "inbox" }, { from: "airbyte", to: "calendars" }, { from: "inbox", to: "reader" }, { from: "reader", to: "sie" },
  { from: "reader", to: "estimator" }, { from: "estimator", to: "singlestore" }, { from: "calendars", to: "planner" }, { from: "planner", to: "sie" },
  { from: "planner", to: "statsig" }, { from: "planner", to: "rules" }, { from: "classifier", to: "sie" }, { from: "classifier", to: "rules" },
  { from: "rules", to: "hub" }, { from: "rules", to: "persona" }, { from: "hub", to: "payments" }, { from: "hub", to: "audit" },
  { from: "audit", to: "singlestore" }, { from: "audit", to: "amplitude" }, { from: "outing", to: "hub" }, { from: "outing", to: "payments" },
  { from: "firedrill", to: "hub" }, { from: "calendars", to: "outing" }, { from: "estimator", to: "planner" }, { from: "outing", to: "cab" }, { from: "outing", to: "transit" },
];

/** Which path a recorded event takes through the scene. Returns null for events with no clear route. */
export function routeOf(e: AgentEvent, house: House): { from: string; to: string } | null {
  const person = house.members.some((m) => m.id === e.actor) ? `p:${e.actor}` : null;
  switch (e.kind) {
    case "airbyte_sync": return { from: "airbyte", to: "hub" };
    case "receipt_ingested": return { from: "inbox", to: "reader" };
    case "line_extracted": return { from: "reader", to: "estimator" };
    case "invite_received": return { from: "calendars", to: "planner" };
    case "model_proposed_basket": return { from: "sie", to: "planner" };
    case "model_failed": return { from: "sie", to: "planner" };
    case "plan_proposed": return { from: "planner", to: "rules" };
    case "plan_auto_approved": return { from: "rules", to: "hub" };
    case "plan_held": return { from: "rules", to: "persona" };
    case "plan_refused": return { from: "rules", to: "hub" };
    case "nothing_to_buy": return { from: "planner", to: "hub" };
    case "decision_gate": return { from: "classifier", to: "rules" };
    case "verification_started": return { from: "hub", to: "persona" };
    case "identity_verified": case "identity_declined": return { from: "persona", to: person ?? "hub" };
    case "approval_blocked_unverified": case "approval_rejected": return { from: person ?? "hub", to: "persona" };
    case "plan_approved": case "plan_skipped": return { from: person ?? "hub", to: "hub" };
    case "order_placed": case "duplicate_prevented": case "payment_failed": case "payment_retried": return { from: "hub", to: "payments" };
    case "stock_corrected": return { from: person ?? "hub", to: "estimator" };
    case "venue_closed": return { from: person ?? "calendars", to: "outing" };
    case "replanned_to_backup": return { from: "outing", to: "payments" };
    case "backup_needs_approval": case "needs_organiser": return { from: "outing", to: "hub" };
    case "outing_proposed": return { from: "calendars", to: "outing" };
    case "agent_rsvp": case "rsvp_changed": return { from: person ?? "calendars", to: "outing" };
    case "outing_priced": return { from: "outing", to: "hub" };
    case "outing_booked": case "booking_changed": return { from: "outing", to: "payments" };
    case "world_call": return { from: "hub", to: e.data.service === "calendar" ? "gcal" : e.data.service === "mail" ? "mailbox" : "webworld" };
    case "chat_message": return { from: "firedrill", to: "hub" };
    case "tool_call": return { from: "hub", to: "rules" };
    case "agent_reply": return { from: "hub", to: "audit" };
    default: return null;
  }
}

export function toneOf(kind: string): Tone {
  if (/refused|failed|declined|blocked|rejected|venue_closed/.test(kind)) return "clay";
  if (/held|verification_started|needs_/.test(kind)) return "ember";
  if (/approved|placed|booked|verified|extracted|prevented|corrected|replanned/.test(kind)) return "sage";
  if (/proposed|priced|gate|sync|ingested|received|rsvp|changed/.test(kind)) return "mist";
  return "ivory";
}

export const TONE_HEX: Record<Tone, string> = { ivory: "#efe9dc", sage: "#8fb08a", ember: "#d9a441", clay: "#cf6f52", mist: "#86a8cb" };

/** Plain-language sender for the activity stream. */
export function senderOf(e: AgentEvent, house: House, nodes: WorldNode[]): { from: string; to: string } {
  const r = routeOf(e, house);
  const name = (id: string) => nodes.find((n) => n.id === id)?.label ?? id;
  return r ? { from: name(r.from), to: name(r.to) } : { from: e.actor === "agent" ? "House agent" : e.actor, to: "Audit log" };
}

/** Did this step touch anything outside the system? In Stocked the answer is always simulated or test mode. */
export function simulationLabel(e: AgentEvent): string | null {
  if (/order_placed|outing_booked|booking_changed|payment_failed|replanned_to_backup/.test(e.kind)) return "TEST MODE";
  if (e.kind === "identity_verified" && String(e.data.provider) === "simulated") return "SIMULATED";
  if (e.kind === "identity_verified" || e.kind === "verification_started") return "SANDBOX";
  if (e.kind === "world_call") return e.data.mode === "firedrill-synthetic" ? "SYNTHETIC" : "SIMULATED";
  return null;
}

/** Counts for the autonomy summary, all derived from recorded plans, outings and orders. */
export function autonomy(house: House) {
  const p = house.plans;
  return {
    automatic: p.filter((x) => x.status === "ordered" && x.decided_by === "agent").length,
    approvedByPerson: p.filter((x) => x.status === "ordered" && x.decided_by !== "agent").length,
    waiting: p.filter((x) => x.status === "held" || x.status === "payment_failed").length,
    refused: p.filter((x) => x.status === "refused").length,
    nothing: p.filter((x) => x.status === "nothing_to_buy").length,
    testOrders: house.orders.filter((o) => !o.plan_id.startsWith("out_")).length,
    testBookings: house.outings.filter((o) => o.status === "booked").length,
    realWorldActions: 0, // every order and booking is test mode; see core/src/payments.rs
  };
}
