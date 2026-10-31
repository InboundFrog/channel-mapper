#!/usr/bin/env python3
"""Regenerate the untested capital-city maps from Free TV Australia's register.

Free TV Australia publishes the DTTB Service Information Register as one PDF
per state group (NSW-ACT, QLD, Vic-SA, NT-WA-Tas), linked from
https://www.freetv.com.au/what-we-do/technology-innovation/ . Each sheet has a
column per market with the ONID/NID/TSID of every multiplex and the SID and LCN
of every service.

    tools/freetv_maps.py reg-NSW-ACT.pdf reg-QLD.pdf reg-Vic-SA.pdf reg-NT-WA-Tas.pdf

Arguments can be URLs; they're downloaded to a temp directory. The NT-WA-Tas
sheet is always needed: Perth's column is joined with maps/au/wa/perth.toml
(tested on a real TV) to number the metro networks. Review `git diff maps/`
afterwards.

Needs Python 3.11+ and pdftotext (poppler-utils).
"""

import argparse
import html
import json
import re
import subprocess
import sys
import tempfile
import tomllib
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PERTH = ROOT / "maps/au/wa/perth.toml"

# File stem -> (market name in the sheet header, state directory, region).
# Once a city's map has been tested on a real TV, remove it here so it isn't
# overwritten.
CITIES = {
    "sydney": ("Sydney", "nsw", "Sydney, NSW"),
    "canberra": ("Canberra", "act", "Canberra, ACT"),
    "melbourne": ("Melbourne", "vic", "Melbourne, Vic"),
    "adelaide": ("Adelaide", "sa", "Adelaide, SA"),
    "brisbane": ("Brisbane", "qld", "Brisbane, Qld"),
    "hobart": ("Hobart", "tas", "Hobart, Tas"),
    "darwin": ("Darwin", "nt", "Darwin, NT"),
}

# Networks run the same way in every metro market. Their services take the
# name and number of the same service (same register LCN) in the Perth map.
METRO = {
    0x1010: "ABC",
    0x3201: "ABC",
    0x3202: "SBS",
    0x1013: "Seven",
    0x1012: "Nine",
    0x1014: "Ten",
}

# Everything else keeps the register's LCN, except these swaps, which put the
# HD copy of the main channel on the main number (the rule the Perth map uses).
SWAP = {
    ("canberra", 0x320C): [(5, 50)],
    ("canberra", 0x3281): [(6, 60)],
    ("canberra", 0x3273): [(8, 80)],
    ("hobart", 0x320B): [(6, 60)],
    ("hobart", 0x320C): [(5, 50)],
    ("hobart", 0x3275): [(8, 80)],
    ("darwin", 0x320B): [(7, 70)],
}

# Section headings in the generated files.
NETWORK = {
    0x1010: "ABC",
    0x3202: "SBS",
    0x1013: "Seven",
    0x1012: "Nine",
    0x1014: "Ten",
    0x320B: "Southern Cross (7)",
    0x320C: "Southern Cross / DDT (10)",
    0x3281: "Prime7",
    0x3273: "WIN (9)",
    0x3275: "WIN (9)",
    0x3220: "Community (C31)",
    0x3234: "Community (44)",
    0x322A: "Community (31)",
    0x32AF: "Larrakia / community",
}

HEX = re.compile(r"^[0-9A-F]{3,4}$")
WORD = re.compile(
    r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="[\d.]+" yMax="([\d.]+)">(.*?)</word>'
)


class Sheet:
    """One register PDF, as rows of (x, text) words.

    Issues are rendered at different scales (Issue 18 is about half the size
    of Issue 16), so every distance is a multiple of the median word height.
    """

    def __init__(self, pdf):
        out = subprocess.run(
            ["pdftotext", "-bbox", str(pdf), "-"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        found = WORD.findall(out)
        heights = sorted(float(y1) - float(y0) for _, y0, y1, _ in found)
        self.h = h = heights[len(heights) // 2]
        words = sorted((float(y), float(x), html.unescape(t)) for x, y, _, t in found)
        self.rows = []
        for y, x, t in words:
            if self.rows and y - self.rows[-1][0] < 0.4 * h:
                self.rows[-1][1].append((x, t))
            else:
                self.rows.append([y, [(x, t)]])
        for _, ws in self.rows:
            ws.sort()

        text = " ".join(t for _, ws in self.rows[:5] for _, t in ws)
        m = re.search(r"Issue:?\s*(\d+)\s+Date\s*:?\s*(\w+ \d{4})", text)
        self.issue = f"Issue {m[1]} ({m[2]})" if m else "unknown issue"

        # Header: "ONID .. TSID" per market, with "NID .. SID" on the same row
        # (Issue 18) or the next (Issue 16).
        self.header = next(
            i for i, (_, ws) in enumerate(self.rows) if sum(t == "ONID" for _, t in ws) > 3
        )
        top = self.rows[self.header][1]
        sub = next(ws for _, ws in self.rows[self.header :] if sum(t == "SID" for _, t in ws) > 3)
        self.onid_x = [x for x, t in top if t == "ONID"]
        self.tsid_x = [x for x, t in top if t == "TSID"]
        self.sid_x = [x for x, t in sub if t == "SID"]
        self.lcn_x = next(x for x, t in top if t == "LCN")
        # "Service Name" heads the label column. It's usually on the
        # "Broadcaster" row, but not always (Issue 18, NT-WA-Tas).
        b = next(i for i, (_, ws) in enumerate(self.rows) if any(t == "Broadcaster" for _, t in ws))
        service_x = min(
            x
            for _, ws in self.rows[max(0, b - 3) : b + 4]
            for x, t in ws
            if t == "Service" and x < self.lcn_x
        )
        self.label_x = service_x - 4 * h  # left of this is the broadcaster column

    def column(self, market):
        """Index of the market's column, from its name in the header."""
        for _, ws in self.rows[: self.header]:
            for x, t in ws:
                if t == market:
                    return max(i for i, ox in enumerate(self.onid_x) if ox <= x + 0.7 * self.h)
        return None

    def services(self, col):
        def at(ws, x):
            hit = [t for wx, t in ws if abs(wx - x) < 3 * self.h and HEX.match(t)]
            return int(hit[0], 16) if hit else None

        mux, out = None, []
        for _, ws in self.rows[self.header + 1 :]:
            lcn = [t for x, t in ws if abs(x - self.lcn_x) < 2 * self.h and t.isdigit()]
            if lcn:
                sid = at(ws, self.sid_x[col])
                if sid is not None and mux:
                    label = " ".join(t for x, t in ws if self.label_x <= x < self.lcn_x - 0.7 * self.h)
                    out.append(dict(onid=mux[0], tsid=mux[1], sid=sid, label=label, lcn=int(lcn[0])))
            elif sum(bool(HEX.match(t)) for _, t in ws) >= 3:
                onid, tsid = at(ws, self.onid_x[col]), at(ws, self.tsid_x[col])
                mux = (onid, tsid) if onid is not None and tsid is not None else None
        return out


def find(sheets, market):
    for s in sheets:
        col = s.column(market)
        if col is not None:
            return s, s.services(col)
    sys.exit(f"no sheet has a column for {market}")


def label_name(label, city, onid):
    if onid == 0x3281 and label.startswith("HD"):
        label = "PRIME7 " + label
    name = re.sub(r"\s*\+?\s*Market Name", " " + city, label)
    return name.replace("Hobart / Launceston", "Hobart").strip()


def build(stem, services, roles):
    city = CITIES[stem][0]
    out = []
    for e in services:
        fam = METRO.get(e["onid"])
        if fam and (fam, e["lcn"]) in roles:
            role = roles[(fam, e["lcn"])]
            if role is None:
                continue  # no longer broadcast in Perth, so probably gone everywhere
            name = re.sub(
                "(?i)perth",
                lambda m: city.upper() if m[0].isupper() else city,
                role["name"],
            )
            number = role.get("number")
        else:
            name = label_name(e["label"], city, e["onid"])
            number = e["lcn"]
            for a, b in SWAP.get((stem, e["onid"]), []):
                number = {a: b, b: a}.get(number, number)
        out.append(dict(e, name=name, number=number))
    numbers = [e["number"] for e in out if e["number"] is not None]
    dup = sorted({n for n in numbers if numbers.count(n) > 1})
    if dup:
        sys.exit(f"{stem}: numbers used twice: {dup}")
    return out


HEADER = """\
# {region} free-to-air DVB-T services -> Australian logical channel numbers.
#
# UNTESTED: not checked against a real TV in {city}. Best effort.
#
# Service IDs (onid/tsid/sid) are from Free TV Australia's DTTB Service
# Information Register, {issue}. Services that started after
# that aren't listed, and IDs may have changed since; tvmap flags both
# ("not in map", "matched by name"), so check the editor before writing.
# Names are a guess at what the TV shows; a different name only raises a flag.
#
# Numbering follows maps/au/wa/perth.toml (tested): where a network has an SD
# and an HD copy of its main channel, the HD copy gets the main number.{abc}
#
# Generated by tools/freetv_maps.py. Once tested on a real TV, take the city
# out of that script so it isn't overwritten.
#
# Per channel:
#   number = N      set the channel number (omit to keep whatever the TV has)
#   hidden = true   remove from the channel list
#   skip   = true   still reachable by typing the number, skipped by CH+/CH-

region = "{region}"
"""

ABC_NOTE = """
# ABC HD and NITV HD aren't in the register, so ABC's SD copy is on 20 and
# NITV's on 36, as in Perth. Give the HD copies 2 and 34 in the editor."""


def render(stem, issue, entries):
    city, _, region = CITIES[stem]
    abc = ABC_NOTE if any(e["onid"] == 0x1010 for e in entries) else ""
    s = HEADER.format(region=region, city=city, issue=issue, abc=abc)
    mux = None
    for e in entries:
        if (e["onid"], e["tsid"]) != mux:
            mux = (e["onid"], e["tsid"])
            head = f"# --- {NETWORK.get(e['onid'], 'Other')} (onid {e['onid']}, tsid {e['tsid']}) "
            s += "\n" + head + "-" * max(3, 75 - len(head)) + "\n"
        s += f"[[channel]]\nonid = {e['onid']}\ntsid = {e['tsid']}\nsid = {e['sid']}\n"
        # JSON escapes (\" \\ \n \uXXXX) are valid in a TOML basic string.
        s += f"name = {json.dumps(e['name'], ensure_ascii=False)}\n"
        if e["number"] is not None:  # no number: keep whatever the TV has
            s += f"number = {e['number']}\n"
        s += "\n"
    return s.rstrip("\n") + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("sheets", nargs="+", help="register PDFs (paths or URLs)")
    ap.add_argument("--check", action="store_true", help="don't write; fail if any map would change")
    args = ap.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        paths = []
        for i, a in enumerate(args.sheets):
            if a.startswith(("http://", "https://")):
                p = Path(tmp) / f"sheet{i}.pdf"
                urllib.request.urlretrieve(a, p)
                paths.append(p)
            else:
                paths.append(Path(a))
        sheets = [Sheet(p) for p in paths]

    perth = {e["sid"]: e for e in tomllib.loads(PERTH.read_text())["channel"]}
    _, perth_services = find(sheets, "Perth")
    roles = {}
    for e in perth_services:
        fam = METRO.get(e["onid"])
        if fam:
            roles[(fam, e["lcn"])] = perth.get(e["sid"])

    changed = []
    for stem, (market, state, _) in CITIES.items():
        sheet, services = find(sheets, market)
        text = render(stem, sheet.issue, build(stem, services, roles))
        path = ROOT / "maps/au" / state / f"{stem}.toml"
        if not path.exists() or path.read_text() != text:
            changed.append(path.relative_to(ROOT))
            if not args.check:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
        print(f"{stem}: {text.count('[[channel]]')} channels ({sheet.issue})")
    if changed:
        print(("would change: " if args.check else "wrote: ") + ", ".join(map(str, changed)))
        if args.check:
            sys.exit(1)


if __name__ == "__main__":
    main()
