"""Superlinked SIE: small task-specific models as tools (extraction, reranking)."""

import os
import re

EXTRACT_MODEL = os.environ.get("SIE_EXTRACT_MODEL", "fastino/gliner2-large-v1")
RERANK_MODEL = os.environ.get("SIE_RERANK_MODEL", "Qwen/Qwen3-Reranker-0.6B")

LINE_SCHEMA = {
    "type": "object",
    "properties": {
        "product": {"type": "string", "description": "name of the product bought, including brand and size"},
        "quantity": {"type": "string", "description": "how many units were bought, such as x2"},
        "price": {"type": "string", "description": "amount of money paid for this line, such as $7.98"},
    },
}

_client = None


def _key() -> str | None:
    return (os.environ.get("SIE_API_KEY") or os.environ.get("SUPERLINKED") or "").strip() or None


def configured() -> bool:
    return _key() is not None


def client():
    global _client
    if _client is None:
        from sie_sdk import SIEClient

        _client = SIEClient(os.environ.get("SIE_BASE_URL", "https://api.superlinked.com"), api_key=_key(), timeout_s=120)
    return _client


def _first(v):
    """GLiNER2 field values may be a string, a list, or {text: ...}."""
    if isinstance(v, list):
        v = v[0] if v else None
    if isinstance(v, dict):
        v = v.get("text") or v.get("value")
    return str(v).strip() if v else None


def extract_lines(text: str) -> list[dict]:
    from sie_sdk.types import Item

    from app import candidate_lines

    out = []
    for i, line in enumerate(candidate_lines(text)):
        reply = client().extract(EXTRACT_MODEL, Item(id=f"l{i}", text=line), output_schema=LINE_SCHEMA)
        data = reply.get("data") or {}
        product, qty_s, price_s = _first(data.get("product")), _first(data.get("quantity")), _first(data.get("price"))
        qty_m = re.search(r"\d+", qty_s or "")
        price_m = re.search(r"\d+(?:\.\d{1,2})?", price_s or "")
        qty = int(qty_m.group()) if qty_m else 1
        line_cents = round(float(price_m.group()) * 100) if price_m else None
        out.append(
            {
                "span": line,  # the exact receipt line; the core verifies it verbatim
                "product": product,
                "qty": qty,
                "line_total_cents": line_cents,
                "unit_price_cents": (line_cents // qty) if line_cents is not None and qty else None,
                # every extracted value must literally appear in the line, or the row is flagged
                "values_in_text": all(v is None or v in line for v in (product, qty_s, price_s)) and product is not None,
            }
        )
    return out


def rerank(query: str, candidates: list[dict]) -> list[dict]:
    from sie_sdk.types import Item

    items = [Item(id=str(c["id"]), text=str(c["text"])) for c in candidates]
    result = client().score(RERANK_MODEL, Item(id="query", text=query), items)
    return [{"id": s["item_id"], "score": float(s["score"]), "rank": int(s["rank"])} for s in result["scores"]]
