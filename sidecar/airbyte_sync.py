"""Airbyte (open source, via PyAirbyte): sync the flat's inbox into a local cache.

The source is Airbyte's `source-file` connector reading two fictional streams:
data/inbox.jsonl (receipt emails) and data/calendars.jsonl (each flatmate's calendar).
Swapping in another Airbyte source (Gmail, Google Sheets, ...) only changes `get_source`.
"""

import os
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = Path(os.environ.get("AIRBYTE_DATA_DIR", ROOT / "data"))
# One Airbyte stream per file: the flat's receipt inbox, and every flatmate's calendar.
STREAMS = {"inbox": DATA / "inbox.jsonl", "calendars": DATA / "calendars.jsonl"}
CACHE_DIR = Path(__file__).resolve().parent / ".airbyte-cache"
FIELDS = ("message_id", "type", "to", "store", "subject", "body", "title", "kind", "venue", "pricing")
INT_FIELDS = ("guests", "nights", "days_from_now")


def configured() -> bool:
    return all(p.exists() for p in STREAMS.values())


def _read(name: str, path: Path) -> list[dict]:
    import airbyte as ab

    source = ab.get_source(
        "source-file",
        config={"dataset_name": name, "format": "jsonl", "url": str(path), "provider": {"storage": "local"}},
        install_if_missing=True,
        use_python="3.10",  # the connector's pinned dependencies do not build on newer Pythons
    )
    source.check()
    source.select_all_streams()
    CACHE_DIR.mkdir(exist_ok=True)
    result = source.read(cache=ab.new_local_cache(cache_name=f"stocked_{name}", cache_dir=str(CACHE_DIR)), force_full_refresh=True)
    out = []
    for row in result.streams[name]:
        r = dict(row)
        rec = {k: r.get(k) for k in FIELDS}
        for k in INT_FIELDS:
            rec[k] = int(r[k]) if r.get(k) is not None else None
        rec["stream"] = name
        rec["airbyte_raw_id"] = str(r.get("_airbyte_raw_id"))
        rec["airbyte_extracted_at"] = str(r.get("_airbyte_extracted_at"))
        out.append(rec)
    return out


def sync() -> dict:
    records = []
    for name, path in STREAMS.items():
        records.extend(_read(name, path))
    return {"connector": "source-file", "streams": list(STREAMS), "records": records}
