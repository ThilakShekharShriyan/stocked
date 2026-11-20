// Starts an identity check: records it in the core, creates a Persona Relay session,
// and returns only the session access token to the browser.
import { NextResponse } from "next/server";
import { CLAIM, CORE, pending, persona } from "@/lib/relay-server";

export async function POST(req: Request) {
  try {
    const { member, plan_id, household } = await req.json();
    const q = household ? `?household=${encodeURIComponent(household)}` : "";
    const started = await fetch(`${CORE}/api/verify/start${q}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ member, plan_id }) });
    const body = await started.json();
    if (!started.ok) return NextResponse.json(body, { status: 400 });
    const { client } = persona();
    const relay = await client.relays.create({ claimType: CLAIM, encryptionKeyJwk: null });
    pending.set(body.verification.id, { relayToken: relay.relayToken, relaySecret: relay.relaySecret, household });
    return NextResponse.json({ verification_id: body.verification.id, access_token: relay.relaySessionAccessToken });
  } catch (e) {
    return NextResponse.json({ error: e instanceof Error ? e.message : String(e) }, { status: 400 });
  }
}
