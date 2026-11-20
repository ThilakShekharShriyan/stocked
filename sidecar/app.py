"""Stocked sidecar: the three integrations that only ship Python SDKs.

- Superlinked SIE: receipt line extraction (GLiNER2) and reranking, via sie-sdk
- Statsig: house rules (auto-approve limit) and agent version
- Airbyte (open source, PyAirbyte): syncs the flat's inbox of receipts and invites

The Rust core calls this over localhost and keeps working on defaults if it is down.
Run: uv run uvicorn app:app --port 8090
"""

import os
import re
import time
from pathlib import Path

from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

load_dotenv(Path(__file__).resolve().parent.parent / ".env.local")


def env(*names: str) -> str | None:
    for n in names:
        v = os.environ.get(n, "").strip()
        if v:
            return v
    return None


import airbyte_sync  # noqa: E402
import sie  # noqa: E402
import statsig_cfg  # noqa: E402

app = FastAPI(title="stocked-sidecar")


@app.get("/health")
def health():
    return {
        "ok": True,
        "sie": sie.configured(),
        "statsig": statsig_cfg.configured(),
        "airbyte": airbyte_sync.configured(),
    }


@app.get("/config")
def config(user: str = "hh_demo"):
    cfg = statsig_cfg.house_config(user)
    if cfg is None:
        raise HTTPException(503, "Statsig not configured")
    return cfg


class ExtractIn(BaseModel):
    text: str


@app.post("/extract")
def extract(body: ExtractIn):
    """Receipt text -> line items. Every returned span is verified verbatim against the text."""
    started = time.time()
    try:
        lines = sie.extract_lines(body.text)
    except Exception as e:  # model cold, key missing, network
        raise HTTPException(503, f"extraction unavailable: {e}")
    return {"lines": lines, "model": sie.EXTRACT_MODEL, "latency_us": int((time.time() - started) * 1e6)}


class RerankIn(BaseModel):
    query: str
    candidates: list[dict]  # [{id, text}]


@app.post("/rerank")
def rerank(body: RerankIn):
    started = time.time()
    try:
        scores = sie.rerank(body.query, body.candidates)
    except Exception as e:
        raise HTTPException(503, f"rerank unavailable: {e}")
    return {"scores": scores, "model": sie.RERANK_MODEL, "latency_us": int((time.time() - started) * 1e6)}


@app.post("/airbyte/sync")
def airbyte_sync_route():
    """Run the Airbyte sync and return the inbox records for the core to ingest."""
    started = time.time()
    try:
        out = airbyte_sync.sync()
    except Exception as e:
        raise HTTPException(503, f"airbyte sync failed: {e}")
    out["latency_us"] = int((time.time() - started) * 1e6)
    return out


# Shared helper used by sie.extract_lines; lives here so it can be unit-checked without the SDK.
PRICE = re.compile(r"\$\s?\d+(?:\.\d{2})?")


def candidate_lines(text: str) -> list[str]:
    """Receipt lines that look like purchases: they contain a price and are not totals."""
    out = []
    for raw in text.splitlines():
        line = raw.strip()
        if PRICE.search(line) and not re.match(r"(?i)^(sub)?total|^tax|^shipping|^paid", line):
            out.append(line)
    return out
