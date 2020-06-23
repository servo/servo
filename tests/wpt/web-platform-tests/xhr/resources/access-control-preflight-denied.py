def main(request, response):
    def fail(message):
        response.content = b"FAIL: " + message
        response.status = 400

    def getState(token):
        server_state = request.server.stash.take(token)
        if not server_state:
            return b"Uninitialized"
        return server_state

    def setState(token, state):
        request.server.stash.put(token, state)

    def resetState(token):
        setState(token, b"")

    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    response.headers.set(b"Access-Control-Max-Age", 1)
    token = request.GET.first(b"token", None)
    state = getState(token)
    command = request.GET.first(b"command", None)

    if command == b"reset":
        if request.method == u"GET":
            resetState(token)
            response.content = b"Server state reset"
        else:
            fail(b"Invalid Method.")
    elif state == b"Uninitialized":
        if request.method == u"OPTIONS":
            response.content = b"This request should not be displayed."
            setState(token, b"Denied")
        else:
            fail(state)
    elif state == b"Denied":
        if request.method == u"GET" and command == b"complete":
            resetState(token)
            response.content = b"Request successfully blocked."
        else:
            setState(token, b"Deny Ignored")
            fail(b"The request was not denied.")
    elif state == b"Deny Ignored":
        resetState(token)
        fail(state)
    else:
        resetState(token)
        fail(b"Unknown Error.")
