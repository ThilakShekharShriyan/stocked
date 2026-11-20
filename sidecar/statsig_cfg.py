"""Statsig: the house rules live in a dynamic config, the agent version in an experiment.

Dynamic config  `stocked_house_rules`   -> { "auto_limit_cents": 4000 }
Experiment      `stocked_agent_version` -> { "agent_version": "v1" | "v2" }
"""

import os

_statsig = None


def _key() -> str | None:
    return (os.environ.get("STATSIG_SDK_KEY") or os.environ.get("STATSIG") or "").strip() or None


def configured() -> bool:
    return _key() is not None


def _client():
    global _statsig
    if _statsig is None:
        from statsig_python_core import Statsig

        _statsig = Statsig(_key())
        _statsig.initialize().wait()
    return _statsig


def house_config(user_id: str) -> dict | None:
    if not configured():
        return None
    from statsig_python_core import StatsigUser

    s = _client()
    user = StatsigUser(user_id)
    rules = s.get_dynamic_config(user, "stocked_house_rules")
    exp = s.get_experiment(user, "stocked_agent_version")
    return {
        "auto_limit_cents": int(rules.get_float("auto_limit_cents", float(os.environ.get("AUTO_LIMIT_CENTS", "4000")))),
        "agent_version": exp.get_string("agent_version", os.environ.get("AGENT_VERSION", "v2")),
        "source": "statsig",
    }
