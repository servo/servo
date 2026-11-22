import json
import time


def main(request, response):
    key = request.GET.first(b"key").decode()
    stash = request.server.stash
    action = request.GET.first(b"action", b"fetch")
    with stash.lock:
        state = stash.take(key) or {
            "count": 0,
            "released": False,
            "status": 200,
            "type": "text/css",
            "headers": [],
        }
        if action == b"release":
            state["released"] = True
        elif action == b"configure":
            state["status"] = int(request.GET.first(b"status", b"200"))
            state["type"] = request.GET.first(b"type", b"text/css").decode()
        elif action == b"fetch":
            state["count"] += 1
            state["headers"].append({
                "referer":
                request.headers.get(b"Referer", b"").decode(),
                "cookie":
                request.headers.get(b"Cookie", b"").decode(),
                "origin":
                request.headers.get(b"Origin", b"").decode(),
            })
        stash.put(key, state)

    headers = [(b"Cache-Control", b"no-store")]
    if b"Origin" in request.headers:
        headers.extend([
            (b"Access-Control-Allow-Origin", request.headers[b"Origin"]),
            (b"Access-Control-Allow-Credentials", b"true"),
        ])
    if action != b"fetch":
        return 200, headers + [(b"Content-Type", b"application/json")
                               ], json.dumps(state)

    # Release is controlled by the test, not by the speed of the network.
    deadline = time.monotonic() + 60
    while not state["released"] and time.monotonic() < deadline:
        time.sleep(0.01)
        with stash.lock:
            state = stash.take(key)
            stash.put(key, state)
    return state["status"], headers + [
        (b"Content-Type", state["type"].encode()),
    ], b"span { color: rgb(0, 0, 255); }"
