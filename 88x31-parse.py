"""Builds 88x31.ddb from 88x31.json, which x227f writes.

Each row of the `pages` table is one button on one page. A child process
streams the rows as newline-delimited JSON into DuckDB's stdin, so they're
never all in memory or written to disk at once.

usage: python3 88x31-parse.py
"""

import json
import os
import subprocess
import sys

import duckdb

COLUMNS = "{page: 'VARCHAR', link: 'VARCHAR', sha256: 'VARCHAR', alt: 'VARCHAR', title: 'VARCHAR', filename: 'VARCHAR'}"


def write_rows(out):
    with open("88x31.json") as f:
        j = json.load(f)
    pages, texts, buttons = j["pages"], j["texts"], j["buttons"]

    def text(i):
        return texts[i] if i is not None else None

    for page, links, bs, alts, titles, filenames in zip(
        pages,
        j["links"],
        j["link_buttons"],
        j["link_button_alts"],
        j["link_button_titles"],
        j["link_button_filenames"],
    ):
        for l, b, ba, bt, bf in zip(links, bs, alts, titles, filenames):
            row = {
                "page": page,
                "link": pages[l] if l is not None else None,
                "sha256": buttons[b],
                "alt": text(ba),
                "title": text(bt),
                "filename": text(bf),
            }
            out.write(json.dumps(row) + "\n")


if sys.argv[1:] == ["--rows"]:
    write_rows(sys.stdout)
    sys.exit()

print("parsing pages")
rows = subprocess.Popen([sys.executable, __file__, "--rows"], stdout=subprocess.PIPE)
os.dup2(rows.stdout.fileno(), sys.stdin.fileno())

db = duckdb.connect("88x31.ddb")
db.sql("DROP TABLE IF EXISTS pages")
db.sql(
    "CREATE TABLE pages AS SELECT * FROM "
    f"read_json('/dev/stdin', format='newline_delimited', columns={COLUMNS})"
)
if rows.wait() != 0:
    sys.exit("writing rows failed")
print(db.execute("SELECT count(*) FROM pages").fetchone()[0], "rows in db")
db.close()
print("parsing pages done!")
