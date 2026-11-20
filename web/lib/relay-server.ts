// Server-only: Persona Relay sessions. The Relay secret never leaves this process.
import { Persona } from "@persona/sdk-node";

export const CLAIM = "live_human_presence";
export const CORE = process.env.CORE_URL ?? "http://localhost:8080";

export function persona() {
  const apiKey = process.env.PERSONA_API_KEY;
  if (!apiKey) throw new Error("PERSONA_API_KEY is not set");
  return { client: new Persona({ apiKey }), apiKey };
}

type Pending = { relayToken: string; relaySecret: string; household?: string };
const g = globalThis as unknown as { __relays?: Map<string, Pending> };
export const pending: Map<string, Pending> = (g.__relays ??= new Map());
