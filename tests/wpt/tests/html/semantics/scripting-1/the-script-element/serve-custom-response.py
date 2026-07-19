# Responds with different content-types and status codes based on
# server-side stash state controlled via query params.
#
# Usage:
# To set the response for future requests with a given key:
#   ?key=token&action=set&code=200&content-type=application/json&network-error=true
#   (code, content-type, and network-error are optional;
#    defaults are 200, application/javascript and false)
# To get the number of times a given key has been requested
#   ?key=token&action=stat
# To get the response for a given key (increments the request count)
#   ?key=token

def main(request, response):
    try:
        stash_key = request.GET.first(b"key", None)
        context = request.server.stash.take(stash_key)

        if not context:
            context = {
                "network_error": False,
                "run_count": 0,
                "status_code": 200,
                "content_type": b"application/javascript",
            }

        action = request.GET.first(b"action", None)
        if action == b"stat":
            request.server.stash.put(stash_key, context)
            response.headers.set(b"Content-Type", b"text/plain")
            response.content = str(context["run_count"])
            return
        elif action == b"reset-stat":
            context["run_count"] = 0
            request.server.stash.put(stash_key, context)
            response.headers.set(b"Content-Type", b"text/plain")
            response.content = "OK"
            return
        elif action == b"set":
            # status code
            code = request.GET.first(b"code", None)
            if code is not None:
                context["status_code"] = int(code)
            # content-type
            content_type = request.GET.first(b"content-type", None)
            if content_type is not None:
                context["content_type"] = content_type
            # network error
            network_error = request.GET.first(b"network-error", None)
            if network_error is not None:
                context["network_error"] = network_error.lower() == b"true"

            request.server.stash.put(stash_key, context)
            response.headers.set(b"Content-Type", b"text/plain")
            response.content = "OK"
            return

        # Simulate a network error if configured
        if context["network_error"]:
            context["run_count"] += 1
            request.server.stash.put(stash_key, context)
            # Simulate a network error by closing the connection without a response.
            # The write call is needed to avoid writing the default headers
            response.writer.write("")
            response.close_connection = True
            return

        # Serve the configured response
        content_type = context["content_type"]
        response.headers.set(b"Content-Type", content_type)
        response.status = context["status_code"]

        if content_type == b"application/javascript":
            response.content = "export default 'hello';"
        elif content_type == b"application/json":
            response.content = '{"hello": "world"}'
        elif content_type == b"text/css":
            response.content = "#test { background-color: rgb(0, 255, 0); }"
        else:
            response.content = "42"

        context["run_count"] += 1
        request.server.stash.put(stash_key, context)
    except Exception as e:
        response.set_error(400, u"Error: %s" % str(e))
