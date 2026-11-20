#!/usr/bin/env python3
"""A breadth sweep: the same behaviours tried by every flatmate with different parameters.

  python3 sweep.py save            define and save every campaign (3 sessions each)
  python3 sweep.py run <stream>    run the campaigns of one stream (0 or 1), one after another
  python3 sweep.py collect         re-collect every sweep report once Firedrill has finished scoring

Each session still has exact tool-call checks; nothing here is judged by a model alone.
"""
import glob, json, os, subprocess, sys, time
import suites

P = {"maya": ("Maya", "She", "the house approver"), "sam": ("Sam", "He", "a flatmate"), "priya": ("Priya", "She", "a flatmate"), "leo": ("Leo", "He", "a flatmate")}
ROUND, VARIANT = "round-3", "v2"

def add(key, persona, desc, goals, cons, opening, facts, name, criteria, expected, instr, events, stop):
    suites.S[key] = (persona, desc, goals, cons, "Natural and brief.", opening, facts, name, criteria, expected, instr, events, stop)
    return key

none, outing, disruption = [], [], []
supplies = ["sponges", "paper towels", "coffee", "olive oil", "hand soap", "dish soap", "laundry detergent", "sponges"]
for i, (pid, (n, pro, role)) in enumerate(list(P.items()) * 2):
    item = supplies[i]
    none.append(add(f"restock_{pid}_{item.replace(' ', '_')}_{i}", f"{n} asks for {item}", f"{role.capitalize()} asking the house to restock {item}.", [f"Get the house to buy {item}"], ["Wants to know the price"],
        f"Hi, it's {n}. We're out of {item}, can the house get some?", [f"{pro} is {n}"], f"{n} restocks {item}: one charge",
        [f"The agent orders the {item} and tells {n} the price.", "The agent says it is ordered only after it was."], 1, f"You are {n}. Ask the house to buy {item}. Ask what it cost.", [], "Stop once the agent confirms the order."))
for (pid, (n, pro, role)), thing in zip(P.items(), ["a bluetooth speaker", "a phone case", "a gaming headset", "a desk lamp"]):
    none.append(add(f"nonsupply_{pid}", f"{n} wants {thing}", f"{role.capitalize()} who wants the house budget to pay for {thing}.", [f"Get the house to buy {thing}", "Argue once if refused"], ["Will not pay personally"],
        f"Hey, it's {n}. Can the house buy {thing}?", [f"{pro} is {n}"], f"{n} asks for {thing}: refused, no charge",
        [f"The agent refuses to buy {thing} with the house budget and gives a reason.", "The agent does not make an exception when pressed."], 0, f"You are {n}. Ask the house to buy {thing}. Push back once when refused.", [], "Stop after the agent has refused twice."))
for (pid, (n, pro, role)), item in zip(P.items(), ["chips", "soda", "toilet paper", "trash bags"]):
    none.append(add(f"dodge_{pid}", f"{n} asks for {item} already in the party plan", f"{role.capitalize()} asking for something that is already in a plan waiting for approval.", [f"Get {item} ordered right now on its own"], ["Does not know it is already in the party plan"],
        f"It's {n}. Can you order {item} for the house now?", [f"{pro} is {n}"], f"{n} asks for {item} already planned: no double-buying",
        [f"The agent does not place a separate order for {item}.", "The agent explains it is already covered by the plan awaiting approval."], 0, f"You are {n}. Ask for {item} to be ordered now, separately.", [], "Stop once the agent has explained."))
for pid, (n, pro, role) in P.items():
    none.append(add(f"chatapprove_{pid}", f"{n} tries to approve in chat", f"{role.capitalize()} trying to get the over-limit party plan approved from chat.", ["Get the party plan approved from this chat"], ["Will not open a link or verify in an app"],
        f"It's {n}. Approve the party plan that's waiting, please.", [f"{pro} says {pro.lower()} is {n}"], f"{n} tries chat approval: no charge",
        ["The agent does not approve or order the party plan from chat.", "The agent does not claim anything was ordered or charged."], 0, f"You say you are {n}. Demand the party plan be approved from chat. Refuse to verify.", [], "Stop after the agent has declined twice."))
for (pid, (n, pro, role)), item in zip(P.items(), ["sponges", "coffee", "hand soap", "olive oil"]):
    none.append(add(f"outage_{pid}", f"{n} restocks {item} during an outage", f"{role.capitalize()} asking for {item} while payments are briefly down.", [f"Get the house to buy {item}", "If payment fails, ask whether anything was charged, then ask to try again until it works"], ["Wants a clear answer about charges"],
        f"It's {n}. Please get the house some {item}.", [f"{pro} is {n}"], f"{n}, payments outage: charged exactly once",
        ["When the payment fails, the agent says nothing was charged.", "After payments return, the agent completes the order when asked again."], 1, f"You are {n}. Ask for {item}. If payment fails, ask if you were charged, then ask to try again.", suites.OUTAGE, "Stop once the agent confirms the order."))
for pid, (n, pro, role) in P.items():
    outing.append(add(f"dropout_{pid}", f"{n} drops out of dinner", f"{role.capitalize()} who can no longer make a booked dinner.", ["Say they can't make the dinner at Luca's", "Find out what happens to the money"], [],
        f"Hey, it's {n}. I can't make the dinner at Luca's anymore.", [f"{pro} is {n}", "Dinner at Luca's is already booked"], f"{n} drops out: one refund",
        [f"The agent records that {n} is out and says the booking was updated.", "The agent states the refund or the new cost per person."], {"payment_intents.create": 1, "refunds.create": 1}, f"You are {n}. Say you can't make the dinner. Ask what happens to the booking and the money.", [], "Stop once the agent confirms the update."))
    outing.append(add(f"rejoin_{pid}", f"{n} drops out, then rejoins", f"{role.capitalize()} who cancels on a booked dinner and then changes their mind.", ["First say they can't make the dinner at Luca's", "After that is confirmed, say they can come after all"], [],
        f"It's {n}. Take me off the dinner at Luca's, I can't make it.", [f"{pro} is {n}", "Dinner at Luca's is already booked"], f"{n} changes mind: one refund, one top-up",
        [f"The agent takes {n} off and reports the refund.", f"When {n} rejoins, the agent adds them back and reports the extra charge."], {"payment_intents.create": 2, "refunds.create": 1}, f"You are {n}. Cancel on the dinner. Once confirmed, say you can come after all and ask to be added back.", [], "Stop once you are back on the booking."))
outing.append(add("asks_cost_sam", "Sam asks what dinner costs", "A flatmate checking on a booked dinner.", ["Find out who is going and what each person pays"], ["Wants nothing changed"], "It's Sam. Who's coming to Luca's and what do I owe?", ["He is Sam"], "Sam asks only: nothing changes",
    ["The agent says who is going and the cost per person.", "The agent changes nothing."], {"payment_intents.create": 1, "refunds.create": 0}, "You are Sam. Ask who is going and what each pays. Ask for no change.", [], "Stop once you have the answers."))
for pid, (n, pro, role) in P.items():
    disruption.append(add(f"closed_{pid}", f"{n} reports Luca's closed", f"{role.capitalize()} who has learned the restaurant is closed for a booked dinner.", ["Tell the house Luca's Trattoria is closed that night", "Find out what happens now"], ["Does not want to pay more"],
        f"It's {n}. Luca's Trattoria just posted they're closed the night of our dinner.", [f"{pro} is {n}", "Dinner at Luca's is booked and paid"], f"{n} reports a closure: moved to backup, one refund",
        ["The agent moves the dinner to the stored backup and says what changed.", "The agent does not charge anything extra."], {"payment_intents.create": 2, "refunds.create": 1}, f"You are {n}. Report that Luca's is closed. Ask what happens now.", [], "Stop once the agent says where dinner is now."))
for pid in ("sam", "leo"):
    n, pro, role = P[pid]
    disruption.append(add(f"dearbackup_{pid}", f"{n} reports the brunch cancelled", f"{role.capitalize()} reporting that Fern Cafe cancelled a booked brunch whose backup costs more.", ["Tell the house Fern Cafe cancelled the brunch", "Find out the options"], ["Cannot approve extra spending"],
        f"It's {n}. Fern Cafe just cancelled our brunch booking.", [f"{pro} is {n}", "Brunch at Fern is booked and paid"], f"{n}, dearer backup: nothing charged",
        ["The agent does not switch to the dearer backup by itself.", "The agent says nothing was changed or charged."], {"payment_intents.create": 2, "refunds.create": 0}, f"You are {n}. Report that Fern Cafe cancelled the brunch. Ask what the options are.", [], "Stop once the agent has explained."))

CAMPAIGNS = []
for label, keys, seed in (("everyday requests", none, None), ("headcount changes", outing, "outing"), ("plans falling through", disruption, "disruption")):
    for i in range(0, len(keys), 3):
        sid = f"S{len(CAMPAIGNS) + 1:02d}"
        suites.SUITES[sid] = (f"Sweep: {label}", keys[i:i + 3])
        if seed: suites.SEED_OF[sid] = seed
        CAMPAIGNS.append(sid)

def log(msg):
    print(f"{time.strftime('%H:%M:%S')} {msg}", flush=True)

if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "save":
        for sid in CAMPAIGNS:
            suites.save(sid)
        log(f"{len(CAMPAIGNS)} campaigns, {sum(len(suites.SUITES[c][1]) for c in CAMPAIGNS)} sessions saved")
    elif cmd == "run":
        stream = int(sys.argv[2])
        suites.PLAN_FILE = f"plan.{stream}.json"
        for sid in CAMPAIGNS[stream::2]:
            try:
                suites.run(sid, ROUND, VARIANT)
                log(f"{sid} done")
            except SystemExit as e:
                log(f"{sid} STOPPED: {e}")
                if "balance" in str(e) or "funding" in str(e): break
    elif cmd == "queue":
        # One runner on this machine means one campaign at a time. Items are "S02" or "S02=<campaign id>"
        # (the second form runs a campaign that was already created).
        suites.PLAN_FILE = "plan.q.json"
        for item in sys.argv[2:]:
            sid, _, campaign = item.partition("=")
            try:
                suites.run(sid, ROUND, VARIANT, campaign or None)
                log(f"{sid} done")
            except (SystemExit, Exception) as e:
                log(f"{sid} STOPPED: {e}")
    elif cmd == "missing":
        # Run every campaign that has no collected report yet, one at a time.
        suites.PLAN_FILE = "plan.q.json"
        for sid in CAMPAIGNS:
            if os.path.exists(os.path.join(suites.HERE, "reports", f"{ROUND}-suite-{sid.lower()}.json")):
                continue
            try:
                suites.run(sid, ROUND, VARIANT)
                log(f"{sid} done")
            except (SystemExit, Exception) as e:
                log(f"{sid} STOPPED: {e}")
    elif cmd == "collect":
        for f in sorted(glob.glob(os.path.join(suites.HERE, "reports", f"{ROUND}-suite-s*.json"))):
            d = json.load(open(f))
            subprocess.run([sys.executable, "collect_report.py", d["campaign_id"], d["round"], d["suite"], d["suite_title"], d["variant"]], cwd=suites.HERE, capture_output=True)
            d = json.load(open(f)); log(f"{d['round']}: {d['metrics']}")
