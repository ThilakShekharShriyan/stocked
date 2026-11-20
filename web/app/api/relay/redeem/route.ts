// After the person finishes the Relay widget: issue a Privacy Pass, redeem the claim,
// and report the result to the core, which records it in the audit trail.
import { NextResponse } from "next/server";
import { CLAIM, CORE, pending, persona } from "@/lib/relay-server";

export async function POST(req: Request) {
  try {
    const { verification_id } = await req.json();
    const p = pending.get(verification_id);
    if (!p) return NextResponse.json({ error: "This check has expired; start it again." }, { status: 400 });
    const { client, apiKey } = persona();
    const { privacyPassToken } = await client.relays.issuePrivacyPass({ claimType: CLAIM });
    const { claimPayload } = await client.relays.generateClaim({ relayToken: p.relayToken, relaySecret: p.relaySecret, privacyPassToken });
    const claim = JSON.parse(claimPayload) as { claim_type: string; claim_result: string; methodology?: { method: string; result: string }[] };
    pending.delete(verification_id);
    const q = p.household ? `?household=${encodeURIComponent(p.household)}` : "";
    const res = await fetch(`${CORE}/api/verify/${verification_id}/attest${q}`, {
      method: "POST",
      headers: { "content-type": "application/json", "x-internal-key": apiKey },
      body: JSON.stringify({ passed: claim.claim_result === "passed", claim_type: claim.claim_type, methodology: claim.methodology ?? [] }),
    });
    return NextResponse.json(await res.json(), { status: res.ok ? 200 : 400 });
  } catch (e) {
    return NextResponse.json({ error: e instanceof Error ? e.message : String(e) }, { status: 400 });
  }
}
