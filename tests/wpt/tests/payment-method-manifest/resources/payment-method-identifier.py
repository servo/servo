"""
Server handler for Payment Method Identifier (PMI) endpoints.

Logs incoming request metadata into request.server.stash under `id`.
Responds with a default Link header pointing to payment-method-manifest.py.
"""

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
            "endpoint": "pmi",
            "step": "pmi",
            "method": request.method,
            "url": request.url,
            "headers": header_dict,
        })
        stash.put(test_id, logs, path=STASH_PATH)

    response.status = 200
    response.headers.set(b"Content-Type", b"text/html")

    test_id_str = test_id.decode("utf-8")
    default_manifest = f"/payment-method-manifest/resources/payment-method-manifest.py?id={test_id_str}"
    response.headers.set(
        b"Link",
        f'<{default_manifest}>; rel="payment-method-manifest"'.encode(
            "utf-8"
        ),
    )

    response.content = b"<!DOCTYPE html><html><body>PMI Endpoint</body></html>"
