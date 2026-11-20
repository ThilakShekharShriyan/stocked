"use client";

// Tests (Firedrill reports) and Insights (Statsig config + funnel by agent version).

import { useEffect, useState } from "react";
import { AgentEvent, House, api, latency, money, timeLabel } from "@/lib/api";
import { Panel } from "./panels";

type Report = { name: string; report: unknown };
export type Session = { session: string; events: AgentEvent[] };

/** One agent conversation: what was said, every tool call with input, output and time, and the code-side scores. */
export function SessionCard({ s }: { s: Session }) {
  const version = String(s.events.find((e) => e.data.agent_version)?.data.agent_version ?? "");
  const tools = s.events.filter((e) => e.kind === "tool_call").length;
  return (
    <details className="rounded-xl border border-stone-200 p-3">
      <summary className="cursor-pointer text-sm">
        <span className="font-semibold">{s.events.find((e) => e.kind === "chat_message")?.summary.slice(0, 70) ?? s.session}</span>
        <span className="ml-2 text-xs text-stone-500">
          {tools} tool call{tools === 1 ? "" : "s"} · agent {version} · {timeLabel(s.events[0]?.ts ?? null)}
        </span>
      </summary>
      <ol className="mt-3 space-y-2">
        {s.events.map((e) => {
          if (e.kind === "chat_message") return <li key={e.id} className="text-sm"><b>Flatmate:</b> {e.summary}</li>;
          if (e.kind === "agent_reply")
            return (
              <li key={e.id} className="text-sm">
                <b>Stocked:</b> {e.summary}
                <span className="ml-2 font-mono text-xs text-stone-500">{e.model} · {latency(e.latency_us)} · {e.cost_credits} credits</span>
              </li>
            );
          if (e.kind === "tool_call")
            return (
              <li key={e.id} className="rounded-lg bg-stone-50 p-2 font-mono text-xs">
                <div className="font-semibold">
                  {e.data.success === false ? "✕" : "✓"} {String(e.data.tool)} <span className="font-normal text-stone-500">· {latency(e.latency_us)}</span>
                </div>
                <div className="text-stone-600">in: {JSON.stringify(e.data.input)}</div>
                <div className="break-words text-stone-600">out: {JSON.stringify(e.data.output).slice(0, 260)}</div>
              </li>
            );
          if (e.kind === "turn_score")
            return (
              <li key={e.id} className="inline-block rounded-full bg-teal-50 px-2 py-0.5 text-xs font-medium text-teal-900 mr-1">
                {String(e.data.score).replaceAll("_", " ")}: {String(e.data.value)}
              </li>
            );
          return null;
        })}
      </ol>
    </details>
  );
}

/** Pull out anything that looks like a pass/fail result from an arbitrary report shape. */
function findResults(v: unknown, path = "", out: { path: string; label: string; passed: boolean; detail: string }[] = []) {
  if (Array.isArray(v)) v.forEach((x, i) => findResults(x, `${path}[${i}]`, out));
  else if (v && typeof v === "object") {
    const o = v as Record<string, unknown>;
    const verdict = o.passed ?? o.pass ?? o.verdict ?? o.outcome ?? o.status;
    const label = (o.name ?? o.key ?? o.title ?? o.evaluation ?? o.persona ?? o.scenario) as string | undefined;
    if (label && (typeof verdict === "boolean" || (typeof verdict === "string" && /^(pass|passed|fail|failed)$/i.test(verdict)))) {
      const passed = verdict === true || (typeof verdict === "string" && /^pass/i.test(verdict));
      const detail = (o.reason ?? o.rationale ?? o.summary ?? o.detail ?? (o.score !== undefined ? `score ${o.score}` : "")) as string;
      out.push({ path, label: String(label), passed, detail: String(detail ?? "") });
    }
    for (const [k, x] of Object.entries(o)) findResults(x, path ? `${path}.${k}` : k, out);
  }
  return out;
}

export function TestsTab() {
  const [reports, setReports] = useState<Report[] | null>(null);
  const [sessions, setSessions] = useState<Session[]>([]);
  useEffect(() => {
    api<{ sessions: Session[] }>("/api/sessions").then((r) => setSessions(r.sessions)).catch(() => setSessions([]));
    api<{ reports: Report[] }>("/api/firedrill/reports").then((r) => setReports(r.reports)).catch(() => setReports([]));
  }, []);
  return (
    <div className="space-y-5">
      <Panel title="Simulated flatmates" hint="Firedrill">
        <p className="max-w-3xl text-base text-stone-700">
          Before a real person relies on it, Stocked is driven by simulated flatmates through the same endpoint and the same checks the phone uses: one approves, one changes their mind, one pushes past the budget, one tries to approve without being
          the approver — with a payments outage mid-conversation. Payment goes to a synthetic Stripe, so “exactly one charge” is counted, not judged.
        </p>
      </Panel>
      <Panel title="Agent sessions" hint="every tool call, with input, output and time">
        {sessions.length === 0 && <p className="text-sm text-stone-500">No conversations yet.</p>}
        <div className="space-y-2">
          {sessions.map((s) => (
            <SessionCard key={s.session} s={s} />
          ))}
        </div>
      </Panel>
      {reports === null && <p className="text-sm text-stone-500">Loading reports…</p>}
      {reports?.length === 0 && (
        <Panel title="No reports yet">
          <p className="text-sm text-stone-600">
            Run a drill and save its report to <code className="font-mono">firedrill/reports/</code> — it will appear here.
          </p>
        </Panel>
      )}
      <div className="grid gap-5 lg:grid-cols-2">
        {reports?.map((r) => {
          const results = findResults(r.report);
          const failed = results.filter((x) => !x.passed).length;
          return (
            <Panel key={r.name} title={r.name} hint={results.length ? `${results.length - failed} passed · ${failed} failed` : undefined}>
              {results.length > 0 && (
                <div className={`mb-3 inline-block rounded-full px-3 py-1 text-sm font-semibold ${failed ? "bg-red-100 text-red-900" : "bg-emerald-100 text-emerald-900"}`}>{failed ? `✕ ${failed} failing` : "✓ All passing"}</div>
              )}
              <ul className="space-y-2">
                {results.slice(0, 14).map((x, i) => (
                  <li key={i} className="flex gap-2 text-sm">
                    <span className={`w-4 shrink-0 font-bold ${x.passed ? "text-emerald-700" : "text-red-700"}`} aria-label={x.passed ? "Passed" : "Failed"}>
                      {x.passed ? "✓" : "✕"}
                    </span>
                    <span>
                      <b>{x.label}</b>
                      {x.detail && <span className="block text-xs text-stone-600">{x.detail}</span>}
                    </span>
                  </li>
                ))}
              </ul>
              <details className="mt-3">
                <summary className="cursor-pointer text-sm font-medium text-teal-800">Raw report</summary>
                <pre className="mt-2 max-h-80 overflow-auto rounded-lg bg-stone-50 p-3 font-mono text-xs">{JSON.stringify(r.report, null, 2)}</pre>
              </details>
            </Panel>
          );
        })}
      </div>
    </div>
  );
}

const STEPS: { label: string; match: (s: string) => boolean }[] = [
  { label: "Plans proposed", match: () => true },
  { label: "Passed checks", match: (s) => s !== "refused" },
  { label: "Approved (auto or by a person)", match: (s) => ["auto_approved", "approved", "ordered", "payment_failed"].includes(s) },
  { label: "Ordered", match: (s) => s === "ordered" },
];

export function InsightsTab({ house }: { house: House }) {
  const [integrations, setIntegrations] = useState<Record<string, Record<string, unknown>> | null>(null);
  useEffect(() => {
    api<Record<string, Record<string, unknown>>>("/api/integrations").then(setIntegrations).catch(() => setIntegrations(null));
  }, []);
  const versions = Array.from(new Set(house.plans.map((p) => p.agent_version))).sort();
  const modelSteps = house.recent_events.filter((e) => e.model);
  return (
    <div className="grid gap-5 lg:grid-cols-2">
      <Panel title="House rules" hint={house.budget.config_source === "statsig" ? "live from Statsig" : "defaults (Statsig not connected)"}>
        <dl className="grid grid-cols-2 gap-4">
          <div>
            <dt className="text-sm text-stone-500">Auto-approve under</dt>
            <dd className="text-2xl font-semibold">{money(house.budget.auto_limit_cents)}</dd>
          </div>
          <div>
            <dt className="text-sm text-stone-500">Agent version</dt>
            <dd className="text-2xl font-semibold">{house.budget.agent_version}</dd>
          </div>
          <div>
            <dt className="text-sm text-stone-500">Monthly budget</dt>
            <dd className="text-2xl font-semibold">{money(house.budget.monthly_cents)}</dd>
          </div>
          <div>
            <dt className="text-sm text-stone-500">Spent this month</dt>
            <dd className="text-2xl font-semibold">{money(house.budget.spent_cents)}</dd>
          </div>
        </dl>
      </Panel>

      <Panel title="Plan → approval → order" hint="by agent version · full funnel in Amplitude">
        {house.plans.length === 0 && <p className="text-sm text-stone-500">No plans yet.</p>}
        {versions.map((v) => {
          const plans = house.plans.filter((p) => p.agent_version === v);
          return (
            <div key={v} className="mb-4">
              <div className="mb-1 text-sm font-semibold">Agent {v}</div>
              {STEPS.map((s) => {
                const n = plans.filter((p) => s.match(p.status)).length;
                return (
                  <div key={s.label} className="mb-1 flex items-center gap-3 text-sm">
                    <span className="w-56 shrink-0 text-stone-600">{s.label}</span>
                    <div className="h-3 flex-1 rounded bg-stone-100">
                      <div className="h-3 rounded bg-teal-700" style={{ width: `${(n / Math.max(plans.length, 1)) * 100}%` }} />
                    </div>
                    <span className="w-6 text-right font-semibold">{n}</span>
                  </div>
                );
              })}
            </div>
          );
        })}
      </Panel>

      <Panel title="Models used" hint="recent steps">
        {modelSteps.length === 0 && <p className="text-sm text-stone-500">No model calls yet.</p>}
        <table className="w-full text-sm">
          <tbody>
            {modelSteps.slice(0, 8).map((e) => (
              <tr key={e.id} className="border-t border-stone-100">
                <td className="py-1.5 pr-2">{e.kind.replaceAll("_", " ")}</td>
                <td className="py-1.5 pr-2 font-mono text-xs">{e.model}</td>
                <td className="py-1.5 pr-2 text-right font-mono text-xs">{(e.latency_us / 1000).toFixed(0)} ms</td>
                <td className="py-1.5 text-right font-mono text-xs">{e.cost_credits} cr</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>

      <Panel title="Connections">
        {!integrations && <p className="text-sm text-stone-500">Checking…</p>}
        {integrations && (
          <ul className="space-y-1.5 text-sm">
            {Object.entries(integrations).map(([name, v]) => {
              const on = v.connected === true || v.configured === true || v.ok === true || name === "payments";
              return (
                <li key={name} className="flex justify-between gap-3">
                  <span className="font-medium capitalize">{name}</span>
                  <span className={on ? "text-emerald-700" : "text-stone-500"}>{name === "payments" ? String(v.provider).replaceAll("_", " ") : on ? (v.sandbox ? "✓ connected (sandbox)" : "✓ connected") : "not connected"}</span>
                </li>
              );
            })}
          </ul>
        )}
      </Panel>
    </div>
  );
}
