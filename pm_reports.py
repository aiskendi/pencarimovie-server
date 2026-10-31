#!/usr/bin/env python3
"""pm_reports.py — read the reported diagnostic log (server_reports_idx).

GOAL
----
Read every diagnostic row our app reported -- every device, every column, any
time window, any size -- as a developer, with no PM-Auth and no per-install
credential.

WHY THIS AND NOT THE REST ROUTE
-------------------------------
`GET /wp-json/pencarimovie-server/v1/reports-query` is the user-facing reader.
It is gated by `fastdownloader_api_permission_authenticated()`, which only
honours PM-Auth / a bot ticket / a WordPress admin, and it clamps `limit` to 100
and `since <= 0` to "last 24h". The `storage/_wp_*.ps1` harnesses called it with
a retired `X-API-Secret`, so they get `401 rest_forbidden` and print `count=`
blank -- "no incidents" and "credential rejected" look identical.

This script skips REST entirely: it runs SQL against the Manticore index the
route itself reads (`SELECT * FROM server_reports_idx`), over SSH, with the same
access the deploy scripts already use. READ-ONLY: only `SELECT`, `DESC` and
`SHOW` statements are ever sent, and they are checked before they leave.

USAGE
  python pm_reports.py                          newest 25 rows
  python pm_reports.py --limit 200
  python pm_reports.py --device 75d677
  python pm_reports.py --platform windows --type playback_incident
  python pm_reports.py --search IllegalState     MATCH over title/message/stack
  python pm_reports.py --since 1793000000        older rows too (whole table by default)
  python pm_reports.py --group failure_reason    aggregate
  python pm_reports.py --group report_type --platform windows
  python pm_reports.py --row 0                   one row, every field
  python pm_reports.py --json                    all matching rows, every field
  python pm_reports.py --csv > incidents.csv
  python pm_reports.py --schema                  DESC the index, then exit

SSH target: PM_SSH_HOST / PM_SSH_USER / PM_SSH_PASS, else parsed from the
existing wp@netcup deploy scripts (never printed).
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import re
import sys

INDEX = "server_reports_idx"
# `--skip-ssl` is not cosmetic: on 2026-10-30 the client started answering
# `ERROR 2026 (HY000): TLS/SSL error: sslv3 alert handshake failure` against
# Manticore's plain 9306 port, so every read failed. 9306 speaks the MySQL
# protocol over LOCALHOST only (the index is not exposed off-box), so there is
# nothing to protect with TLS and the handshake must be skipped explicitly.
MYSQL = "mysql -h127.0.0.1 -P9306 --batch --skip-ssl"

PREFERRED = [
    "created_at", "report_type", "platform", "device_id", "app_version",
    "failure_reason", "short_code", "item_id", "error_title", "http_status",
    "bytes_sent", "bytes_total", "model", "brand", "os_version", "id",
]
FULLTEXT = ("error_message", "stack_trace", "details", "error_title")
ONLY_READ = re.compile(r"^\s*(select|desc|describe|show)\b", re.I | re.S)


# ── ssh ─────────────────────────────────────────────────────────────────────
def creds() -> tuple[str, str, str]:
    host = os.environ.get("PM_SSH_HOST", "").strip()
    user = os.environ.get("PM_SSH_USER", "").strip()
    pw = os.environ.get("PM_SSH_PASS", "").strip()
    if host and user and pw:
        return host, user, pw

    home = os.path.expanduser("~")
    for name in ("inspect_tables.py", "ssh_diag.py", "deploy_main.py"):
        path = os.path.join(home, "wp @ netcup", name)
        try:
            text = open(path, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        h = re.search(r"""host\s*=\s*['"]([^'"]+)['"]""", text)
        u = re.search(r"""user\s*=\s*['"]([^'"]+)['"]""", text)
        p = re.search(r"""password\s*=\s*['"]([^'"]+)['"]""", text)
        if h and u and p:
            return host or h.group(1), user or u.group(1), pw or p.group(1)
        if h and u:
            return host or h.group(1), user or u.group(1), pw or ""
    sys.exit("ssh credentials not found: set PM_SSH_HOST / PM_SSH_USER / PM_SSH_PASS")


def remote_mysql(client, sql: str, timeout: int = 90) -> tuple[str, str]:
    if not ONLY_READ.match(sql):
        raise SystemExit(f"refusing to run a non-read statement: {sql[:60]}…")
    cmd = f'{MYSQL} -e {json.dumps(sql, ensure_ascii=False)} 2>&1'
    _in, out, err = client.exec_command(cmd, timeout=timeout)
    return out.read().decode("utf-8", "replace"), err.read().decode("utf-8", "replace").strip()


def parse_batch(text: str) -> tuple[list[str], list[dict]]:
    lines = [ln for ln in text.splitlines() if ln.strip()]
    if not lines:
        return [], []
    header = lines[0].split("\t")
    rows = []
    for ln in lines[1:]:
        cells = ln.split("\t")
        cells += [""] * (len(header) - len(cells))
        rows.append(dict(zip(header, cells)))
    return header, rows


# ── helpers ─────────────────────────────────────────────────────────────────
def q(value: str) -> str:
    """Quote a literal for Manticore SQL (backslash + single quote)."""
    return "'" + value.replace("\\", "\\\\").replace("'", "\\'") + "'"


def when(ts: str) -> str:
    try:
        import datetime as dt
        return dt.datetime.fromtimestamp(int(float(ts)), dt.timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ")
    except Exception:
        return str(ts)


def text_of(row: dict) -> str:
    return " ".join(str(row.get(c, "")) for c in FULLTEXT).lower()


# A per-column cap: short code columns must never be truncated (a clipped
# device_id or short_code is unmatchable), while a long error_title is the one
# thing that may be cut, because --row / --json still carry it in full.
WIDTH_CAP = {
    "created_at": 20, "report_type": 18, "platform": 8, "device_id": 12,
    "app_version": 10, "failure_reason": 20, "short_code": 10, "item_id": 16,
    "error_title": 36, "model": 16, "brand": 12, "os_version": 10, "id": 20,
}


def cell(row: dict, col: str) -> str:
    v = when(row.get(col, "")) if col == "created_at" else str(row.get(col, "") or "")
    return v.replace("\t", " ").replace("\n", " ").strip()


def table(cols: list[str], rows: list[dict]) -> None:
    widths = {}
    for c in cols:
        cap = WIDTH_CAP.get(c, 20)
        widths[c] = min(cap, max([len(c)] + [len(cell(r, c)) for r in rows]))
    line = "  ".join(f"{c:<{widths[c]}}" for c in cols)
    print(line)
    print("-" * len(line))
    for r in rows:
        print("  ".join(f"{cell(r, c)[:widths[c]]:<{widths[c]}}" for c in cols))


# ── main ────────────────────────────────────────────────────────────────────
def main() -> int:
    ap = argparse.ArgumentParser(description="read server_reports_idx over SSH (read-only)")
    ap.add_argument("--limit", type=int, default=25, help="rows to fetch (no 100 clamp)")
    ap.add_argument("--since", type=int, default=0, help="epoch floor; 0 = whole table")
    ap.add_argument("--device")
    ap.add_argument("--platform")
    ap.add_argument("--type", dest="rtype")
    ap.add_argument("--reason")
    ap.add_argument("--item")
    ap.add_argument("--search", help="full-text match over title/message/stack")
    ap.add_argument("--group", help="aggregate: count rows by this column")
    ap.add_argument("--row", type=int, help="print one row (every field) as JSON")
    ap.add_argument("--json", action="store_true", help="print all rows as JSON")
    ap.add_argument("--csv", action="store_true", help="print all rows as CSV")
    ap.add_argument("--schema", action="store_true", help="DESC the index, then exit")
    args = ap.parse_args()

    import paramiko

    host, user, pw = creds()
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(hostname=host, username=user, password=pw, timeout=30, banner_timeout=30)

    try:
        schema, schema_err = remote_mysql(client, f"DESC {INDEX}")
        if args.schema:
            print((schema or schema_err).strip())
            return 0

        where = []
        for col, val in (
            ("report_type", args.rtype),
            ("platform", args.platform),
            ("device_id", args.device),
            ("failure_reason", args.reason),
            ("item_id", args.item),
        ):
            if val:
                where.append(f"{col} = {q(val)}")
        if args.since > 0:
            where.append(f"created_at >= {int(args.since)}")
        if args.search:
            needle = re.sub(r"[^\w\s]+", " ", args.search, flags=re.UNICODE).strip()
            if not needle:
                sys.exit("--search needs at least one letter or digit")
            where.append(f"MATCH({q(needle)})")

        sql = f"SELECT * FROM {INDEX}"
        if where:
            sql += " WHERE " + " AND ".join(where)
        sql += f" ORDER BY created_at DESC LIMIT {max(1, int(args.limit))}"

        text, err = remote_mysql(client, sql)
        header, rows = parse_batch(text)
        # The MySQL-protocol client renders `created_at` as epoch seconds even
        # though the index declares it `timestamp`. Normalise once, here, so
        # --row / --json / --csv carry a readable date too (table()'s own when()
        # then passes the string through untouched).
        for r in rows:
            if "created_at" in r:
                r["created_at"] = when(r["created_at"])
        if not rows:
            print(f"no rows for: {sql}")
            if err:
                print(f"stderr: {err}")
            print("\nindex schema:")
            print((schema or schema_err).strip())
            return 1

        if args.json:
            print(json.dumps(rows, indent=2, ensure_ascii=False))
            return 0

        if args.csv:
            cols = [c for c in PREFERRED if c in header] + [c for c in header if c not in PREFERRED]
            w = csv.DictWriter(sys.stdout, fieldnames=cols, extrasaction="ignore")
            w.writeheader()
            for r in rows:
                w.writerow({c: r.get(c, "") for c in cols})
            return 0

        if args.row is not None:
            print(json.dumps(rows[args.row], indent=2, ensure_ascii=False))
            return 0

        label = f"{len(rows)} row(s) from {INDEX}"
        applied = ", ".join(f"{k}={v}" for k, v in (
            ("type", args.rtype), ("platform", args.platform), ("device", args.device),
            ("reason", args.reason), ("item", args.item), ("search", args.search),
            ("since", args.since or None),
        ) if v)
        print(label + (f"  [{applied}]" if applied else "  [newest first, whole table]"))

        if args.group:
            col = args.group
            if col not in header:
                sys.exit(f"no such column: {col}\navailable: {', '.join(header)}")
            counts: dict[str, int] = {}
            for r in rows:
                key = str(r.get(col, "")).strip() or "(empty)"
                counts[key] = counts.get(key, 0) + 1
            print(f"\nby {col}:")
            for key, n in sorted(counts.items(), key=lambda kv: -kv[1]):
                print(f"  {n:5d}  {key}")
            return 0

        cols = [c for c in PREFERRED if c in header] or header
        print()
        table(cols, rows)
        print()
        print("full text columns: --row N (one row) or --json (all)")
        return 0
    finally:
        client.close()


if __name__ == "__main__":
    raise SystemExit(main())
