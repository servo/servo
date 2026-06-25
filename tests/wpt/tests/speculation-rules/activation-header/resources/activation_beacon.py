# Copyright 2026 The Chromium Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import json
import os


def main(request, response):
    try:
        action = request.GET.get(b"action", b"serve")
        key = request.GET.get(b"uuid")

        if not key:
            return 400, [(b"Content-Type", b"text/plain")], b"Missing uuid"

        # Ensure key is string
        key_str = key.decode('utf-8') if isinstance(key, bytes) else key
        action_str = action.decode('utf-8') if isinstance(action,
                                                          bytes) else action

        state = request.server.stash.take(key) or {}

        if action_str == "serve":
            beacon_url = f"activation_beacon.py?uuid={key_str}&action=beacon"
            headers = [(b"Content-Type", b"text/html"),
                       (b"Cache-Control", b"no-store"),
                       (b"on-prefetch-activation", beacon_url.encode("utf-8"))]

            state["fetched"] = True
            request.server.stash.put(key, state)

            # Read executor.sub.html from prefetch/resources/
            path = os.path.join(os.path.dirname(__file__),
                                "..", "..", "prefetch", "resources", "executor.sub.html")
            from wptserve.pipes import template
            content = template(request, open(path, "rb").read())

            return headers, content

        elif action_str == "beacon":
            if request.method != b"HEAD" and request.method != "HEAD":
                state["error"] = f"Method not allowed: {request.method}"
                request.server.stash.put(key, state)
                return 405, [(b"Content-Type", b"text/plain")
                             ], b"Method not allowed"

            state["beacon_received"] = True
            request.server.stash.put(key, state)

            return 204, [(b"Content-Type", b"text/plain")], b""

        elif action_str == "check":
            request.server.stash.put(key, state)  # Put it back
            headers = [(b"Content-Type", b"application/json"),
                       (b"Cache-Control", b"no-store")]
            return headers, json.dumps(state).encode("utf-8")

        else:
            state["error"] = f"Invalid action: {action_str}"
            request.server.stash.put(key, state)
            return 400, [(b"Content-Type", b"text/plain")], b"Invalid action"

    except Exception as e:
        raise
