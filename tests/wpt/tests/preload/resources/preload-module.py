import json
import time


# Serves a JavaScript module for modulepreload tests, letting the test control
# when each module is served.
#
# Query parameters:
#   uuid=<uuid>     Names the request so that the test can block and release it.
#                   Required by all of the below.
#   block=1         Block the response until a release=1 request with the same
#                   uuid arrives (used to keep a module outstanding while the
#                   test observes something else).
#   release=1       Release the blocked request with the same uuid.
#   import=<url>    Emit `import "<url>";` so the module has a dependency.
#   syntaxerror=1   Emit a module with a parse error.
#   (otherwise)     Emit an empty module.
#
# Only release=1 ever writes to the stash and only a blocked request ever reads
# from it, so there is no read-modify-write to race over.
def main(request, response):
    params = request.GET
    uuid = params.first(b"uuid").decode()

    if b"release" in params:
        try:
            request.server.stash.put(uuid, True)
        except Exception:
            # Already released and not yet taken; nothing to do.
            pass
        return (200, [(b"Content-Type", b"text/javascript"),
                      (b"Cache-Control", b"no-store")], b"")

    if b"block" in params:
        remaining = 300  # ~30s safety timeout
        while remaining > 0 and not request.server.stash.take(uuid):
            time.sleep(0.1)
            remaining -= 1

    headers = [(b"Content-Type", b"text/javascript"),
               (b"Cache-Control", b"no-store")]

    if b"syntaxerror" in params:
        return (200, headers, b"if (")

    if b"import" in params:
        dep = params.first(b"import").decode()
        return (200, headers, ("import " + json.dumps(dep) + ";\n").encode())

    return (200, headers, b"export default 1;\n")
