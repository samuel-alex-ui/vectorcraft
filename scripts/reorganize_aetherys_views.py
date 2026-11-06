#!/usr/bin/env python3
"""Reorganize the Aetherys map hierarchy without modifying drawing elements."""
import json
from pathlib import Path

path = Path("maps/Aetherys World Map2.vectorcraft")
data = json.loads(path.read_text(encoding="utf-8"))
document = data["document"]
existing = document["layers"]
if any(layer["name"].startswith("Ansicht ") for layer in existing):
    raise SystemExit("Already organized; no changes needed")
next_id = max(document.get("next_id", 1), max(layer["id"] for layer in existing) + 1)
def group(name, children=()):
    global next_id
    result = {"id": next_id, "name": name, "kind": {
        "type": "layer", "color": {"Preset": 2}, "template": False,
        "printable": True, "children": list(children)}}
    next_id += 1
    return result

colored = {
    "Küsten · Kontinentalschelf und Lagunen",
    "Kontinente und Inselwelten",
    "Biomzonen · flächendeckende organische Regionen",
}
coastline = [layer for layer in existing if layer["name"] == "Küstenlinie"]
contours = [layer for layer in existing if layer["name"] not in colored and layer not in coastline]
geography = [layer for layer in existing if layer["name"] in colored]
def view(n, contours=(), geography=()):
    return group(f"Ansicht {n}", [
        group("Kontur und Meere", contours),
        group("Geografie", geography),
        group("Politik"),
        group("Wirtschaft"),
    ])
document["layers"] = [
    view(1, contours, geography),
    view(2, coastline),
    view(3), view(4), view(5),
]
document["next_id"] = next_id
assert len(contours) + len(geography) + len(coastline) == len(existing)
assert {x["id"] for x in contours + geography + coastline} == {x["id"] for x in existing}
path.write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
print(f"Reorganized {len(existing)} layers; coastline layers found: {len(coastline)}")
