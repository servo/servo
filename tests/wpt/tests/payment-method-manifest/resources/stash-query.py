"""
Server handler for inspecting server access logs.

Given an `id` query parameter, this handler reads and returns the JSON list of
recorded request objects (containing endpoint, method, url, and headers) logged
for that specific test run.

Supports CORS (Access-Control-Allow-Origin: *) to allow cross-origin test
scripts to query server access logs.
"""

import json
import os
import sys

resources_dir = os.path.dirname(__file__)
if resources_dir not in sys.path:
    sys.path.insert(0, resources_dir)

from server_constants import STASH_PATH


def main(request, response):
    test_id = request.GET.get(b"id")
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    if not test_id:
        response.status = 400
        response.content = json.dumps({"error": "missing id"}).encode("utf-8")
        return

    stash = request.server.stash
    with stash.lock:
        logs = stash.take(test_id, path=STASH_PATH) or []
        if logs:
            # Preserve logs in stash for potential subsequent assertions
            stash.put(test_id, logs, path=STASH_PATH)

    response.status = 200
    response.content = json.dumps(logs).encode("utf-8")
