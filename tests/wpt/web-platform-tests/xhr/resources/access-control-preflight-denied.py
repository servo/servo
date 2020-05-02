def main(request, response):
    def fail(message):
        response.content = "FAIL: " + str(message)
        response.status = 400

    def getState(token):
        server_state = request.server.stash.take(token)
        if not server_state:
            return "Uninitialized"
        return server_state

    def setState(token, state):
        request.server.stash.put(token, state)

    def resetState(token):
        setState(token, "")

    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
    response.headers.set("Access-Control-Max-Age", 1)
    token = request.GET.first("token", None)
    state = getState(token)
    command = request.GET.first("command", None)

    if command == "reset":
        if request.method == "GET":
            resetState(token)
            response.content = "Server state reset"
        else:
            fail("Invalid Method.")
    elif state == "Uninitialized":
        if request.method == "OPTIONS":
            response.content = "This request should not be displayed."
            setState(token, "Denied")
        else:
            fail(state)
    elif state == "Denied":
        if request.method == "GET" and command == "complete":
            resetState(token)
            response.content = "Request successfully blocked."
        else:
            setState(token, "Deny Ignored")
            fail("The request was not denied.")
    elif state == "Deny Ignored":
        resetState(token)
        fail(state)
    else:
        resetState(token)
        fail("Unknown Error.")
