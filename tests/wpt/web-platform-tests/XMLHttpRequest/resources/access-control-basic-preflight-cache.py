def main(request, response):
    def fail(message):
        response.content = "FAIL " + request.method + ": " + str(message)
        response.status = 400

    def getState(token):
        server_state = request.server.stash.take(token)
        if not server_state:
            return "Uninitialized"
        return server_state

    def setState(state, token):
        request.server.stash.put(token, state)

    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
    response.headers.set("Access-Control-Allow-Credentials", "true")
    token = request.GET.first("token", None)
    state = getState(token)

    if state == "Uninitialized":
        if request.method == "OPTIONS":
            response.headers.set("Access-Control-Allow-Methods", "PUT")
            response.headers.set("Access-Control-Max-Age", 10)
            setState("OPTIONSSent", token)
        else:
            fail(state)
    elif state == "OPTIONSSent":
        if request.method == "PUT":
            response.content = "PASS: First PUT request."
            setState("FirstPUTSent", token)
        else:
            fail(state)
    elif state == "FirstPUTSent":
        if request.method == "PUT":
            response.content = "PASS: Second PUT request. Preflight worked."
        elif request.method == "OPTIONS":
            response.headers.set("Access-Control-Allow-Methods", "PUT")
            setState("FAILSecondOPTIONSSent", token)
        else:
            fail(state)
    elif state == "FAILSecondOPTIONSSent":
        if request.method == "PUT":
            fail("Second OPTIONS request was sent. Preflight failed.")
        else:
            fail(state)
    else:
        fail(state)
