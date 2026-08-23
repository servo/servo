"""
Server handler for Payment Method Manifest (PMM) JSON files.

Logs incoming request metadata into request.server.stash under `id`.
Responds with a default JSON manifest body containing supported_origins.
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
    if not test_id:
        response.status = 400
        response.headers.set(b"Content-Type", b"text/plain")
        response.content = b"Missing required 'id' query parameter"
        return

    # Record incoming HTTP request metadata into server stash for test verification
    stash = request.server.stash
    with stash.lock:
        logs = stash.take(test_id, path=STASH_PATH) or []
        header_dict = {
            k.decode("utf-8").lower(): b", ".join(v).decode("utf-8")
            for k, v in request.headers.items()
        }
        logs.append({
            "endpoint": "payment-method-manifest",
            "step": "payment-method-manifest",
            "method": request.method,
            "url": request.url,
            "headers": header_dict,
        })
        stash.put(test_id, logs, path=STASH_PATH)

    response.status = 200
    response.headers.set(b"Content-Type", b"application/json")

    # Respond with a basic payment method manifest that does not contain a
    # pointer to a web-app manifest, which is fine for the current tests.
    origin = f"{request.url_parts.scheme}://{request.url_parts.netloc}"
    response.content = json.dumps({"supported_origins": [origin]}).encode(
        "utf-8"
    )
