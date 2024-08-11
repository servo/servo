"""Test reporting origin server used for two reasons:

  1. It is a workaround for lack of preflight support in the test server.
  2. Stashes requests so they can be inspected by tests.
"""

from wptserve.stash import Stash
import json

REQUESTS = "9250f93f-2c05-4aae-83b9-2817b0e18b4d"


headers = [
    b"attribution-reporting-eligible",
    b"attribution-reporting-support",
    b"referer",
]


def store_request(request) -> None:
    obj = {
        "method": request.method,
        "url": request.url,
    }
    for header in headers:
        value = request.headers.get(header)
        if value is not None:
            obj[str(header, "utf-8")] = str(value, "utf-8")
    with request.server.stash.lock:
        requests = request.server.stash.take(REQUESTS)
        if not requests:
            requests = []
        requests.append(obj)
        request.server.stash.put(REQUESTS, requests)
    return None


def get_requests(request) -> str:
    with request.server.stash.lock:
        return json.dumps(request.server.stash.take(REQUESTS))


def main(request, response):
    """
    For most requests, simply returns a 200. Actual source/trigger registration
    headers are piped using the `pipe` query param.

    If a `clear-stash` param is set, it will clear the stash.
    """
    if request.GET.get(b"clear-stash"):
        request.server.stash.take(REQUESTS)
        return

    # We dont want to redirect preflight requests. The cors headers are piped
    # so we can simply return a 200 and redirect the following request
    if request.method == "OPTIONS":
        response.status = 200
        return

    if request.GET.get(b"get-requests"):
        return get_requests(request)

    if request.GET.get(b"store-request"):
        store_request(request)
        return ""
