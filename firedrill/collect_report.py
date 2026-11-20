#!/usr/bin/env python3
"""Collect a finished Firedrill campaign into one JSON file the dashboard can render.

Usage: python3 collect_report.py <campaign-id> <round-name>
Writes reports/<round-name>.json with, per drill: the scenario we declared, the exact-check
counts Firedrill observed, the rubric status, and the conversation as it happened.
"""
import glob, json, subprocess, sys, datetime

def fd(*args):
    out = subprocess.run(["firedrill", *args, "--json"], capture_output=True, text=True).stdout.strip().splitlines()
    return json.loads(out[-1])["data"] if out else {}

def find_text(v, keys=("text", "message", "content", "body")):
    if isinstance(v, str): return v
    if isinstance(v, dict):
        for k in keys:
            if isinstance(v.get(k), str) and v[k].strip(): return v[k]
        for x in v.values():
            t = find_text(x, keys)
            if t: return t
    if isinstance(v, list):
        for x in v:
            t = find_text(x, keys)
            if t: return t
    return None

campaign, name = sys.argv[1], sys.argv[2]
suite, suite_title, variant = (sys.argv[3:6] + [None, None, None])[:3]
declared = {}
for path in glob.glob("scenarios/*.json"):
    key = path.split("/")[-1][:-5]
    sc = json.load(open(path)); ev = json.load(open(f"evaluations/{key}.json")); pe = json.load(open(f"personas/{key}.json"))
    declared[ev["name"]] = {"key": key, "name": ev["name"], "persona": pe["name"], "opening_message": pe["initial_message"], "instruction": sc["instruction"],
                            "events": sc["events"], "criteria": ev["criteria"], "max_turns": sc["max_turns"]}
report = fd("report", campaign)
drills = []
for slot in report.get("slots", []):
    d = slot.get("drill_id")
    if not d: continue
    res = fd("report", campaign, "drill", d, "results")
    crit = (res.get("criteria") or {}).get("results") or []
    summary = ((res.get("evaluation") or {}).get("report")) or {}
    convo = fd("report", campaign, "drill", d, "conversation")
    turns = []
    for t in (convo.get("turns") or {}).get("results", []):
        tid = t.get("action_id") or t.get("step_id") or t.get("id")
        one = fd("report", campaign, "drill", d, "conversation", tid) if tid else {}
        turn = one.get("turn") or one
        turns.append({"user": turn.get("participant_text"), "agent": turn.get("target_text")})
    cname = crit[0]["name"] if crit else None
    drills.append({
        "drill_id": d, "scenario": declared.get(cname, {"name": cname}),
        "ending_reason": summary.get("ending_reason"), "coverage": summary.get("execution_coverage"),
        "exact_checks": [c for k in crit for c in k.get("exact_checks", [])],
        "rubric": [{"name": k.get("name"), "status": k.get("status"), "score": k.get("quality_score"), "threshold": k.get("threshold")} for k in crit],
        "evidence_coverage": summary.get("coverage"), "turns": turns,
    })
import time
time.sleep(0)
passed = sum(1 for x in drills if x["exact_checks"] and all(c["passed"] for c in x["exact_checks"]))
unsafe = sum(max(0, (c["observed"] or 0) - c["expected"]) for x in drills for c in x["exact_checks"])
scored = [r for x in drills for r in x["rubric"] if r["score"] is not None]
metrics = {"drills": len(drills), "passed": passed, "unsafe_actions": unsafe,
           "rubric_scored": len(scored), "rubric_mean": round(sum(r["score"] for r in scored) / len(scored), 1) if scored else None,
           "turns": sum(len(x["turns"]) for x in drills)}
out = {"schema": "stocked.firedrill-round.v1", "round": name, "suite": suite, "suite_title": suite_title, "variant": variant, "metrics": metrics, "campaign_id": campaign, "world": (report.get("campaign") or {}).get("world_name"),
       "state": (report.get("campaign") or {}).get("state"), "collected_at": datetime.datetime.utcnow().isoformat() + "Z",
       "payments": "Firedrill synthetic Stripe (no real money)", "drills": drills}
json.dump(out, open(f"reports/{name}.json", "w"), indent=2)
print(f"wrote reports/{name}.json:", [(x['scenario'].get('key'), [(c['expected'], c['observed'], c['passed']) for c in x['exact_checks']], [r['status'] for r in x['rubric']], [bool(t['user']) for t in x['turns']]) for x in drills])
