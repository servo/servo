"""Integration harness for the in-process Severin Python bridge.

This is ordinary Python test source rather than an internal Rust queue unit test:
it imports the built extension, loads a real local page, waits for the load phase
to finish, pumps the owner thread, reads the first JavaScript-originated frame,
writes a reply, pumps again, and expects a second JavaScript-originated frame
proving the first Promise resolved inside the page.
"""

from __future__ import annotations

import json
from pathlib import Path

import severin


FIXTURE = Path(__file__).resolve().parent.parent / "fixtures" / "bridge-roundtrip" / "index.html"


def pump_until_frame(app: severin.App, limit: int = 10_000):
    for _ in range(limit):
        app.pump()
        frame = app.read()
        if frame is not None:
            return frame
    raise AssertionError("timed out waiting for a Severin bridge frame")


def test_javascript_python_json_roundtrip():
    app = severin.App(width=800, height=600)
    try:
        app.load_path(str(FIXTURE))
        app.run()

        receipt, json_text = pump_until_frame(app)
        assert json.loads(json_text) == ["request", {"hello": "world"}]

        app.write(receipt, '{"ok":true}')

        _, resolved_json_text = pump_until_frame(app)
        assert json.loads(resolved_json_text) == ["resolved", {"ok": True}]
    finally:
        app.close()


if __name__ == "__main__":
    test_javascript_python_json_roundtrip()
