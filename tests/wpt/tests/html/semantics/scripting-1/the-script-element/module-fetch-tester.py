# Test helper for module fetch caching tests (whatwg/html#10327).
#
# Serves a JavaScript module and tracks how many times each key has been
# requested, so tests can assert how many fetches actually reached the server.
# It needs no separate configuration request, so a re-import of the same
# specifier can be started synchronously from a rejection/error handler (while
# draining the microtask queue).
#
# Query parameters:
#   key=<uuid>          Required. Identifies an independent request counter.
#   action=stat         Return the request count for key as text/plain, without
#                       counting this request or serving a module.
#   mode=always-fail    Respond to every request with an HTTP 404.
#   mode=fail-first     (default) Respond to the first request for key with an
#                       HTTP 404, and serve a JS module for every later request.
#
# All requests except action=stat increment the counter.

def main(request, response):
    key = request.GET.first(b"key")

    if request.GET.first(b"action", None) == b"stat":
        with request.server.stash.lock:
            run_count = request.server.stash.take(key)
            if run_count is None:
                run_count = 0
            request.server.stash.put(key, run_count)
        response.headers.set(b"Content-Type", b"text/plain")
        response.content = str(run_count)
        return

    with request.server.stash.lock:
        run_count = request.server.stash.take(key)
        if run_count is None:
            run_count = 0
        run_count += 1
        request.server.stash.put(key, run_count)

    mode = request.GET.first(b"mode", b"fail-first")
    should_fail = mode == b"always-fail" or (mode == b"fail-first" and run_count == 1)

    if should_fail:
        # Fail with an HTTP error, which must not be cached.
        response.status = 404
        response.headers.set(b"Content-Type", b"text/plain")
        response.content = b"not found"
        return

    response.headers.set(b"Content-Type", b"application/javascript")
    response.content = b"export default 'hello';"
