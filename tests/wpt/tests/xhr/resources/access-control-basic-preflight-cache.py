
from wptserve.utils import isomorphic_encode

def main(request, response):
    def fail(message):
        response.content = b"FAIL " + isomorphic_encode(request.method) + b": " + message
        response.status = 400

    def getState(token):
        server_state = request.server.stash.take(token)
        if not server_state:
            return b"Uninitialized"
        return server_state

    def setState(state, token):
        request.server.stash.put(token, state)

    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    response.headers.set(b"Access-Control-Allow-Credentials", b"true")
    token = request.GET.first(b"token", None)
    state = getState(token)

    if state == b"Uninitialized":
        if request.method == u"OPTIONS":
            response.headers.set(b"Access-Control-Allow-Methods", b"PUT")
            response.headers.set(b"Access-Control-Max-Age", 10)
            setState(b"OPTIONSSent", token)
        else:
            fail(state)
    elif state == b"OPTIONSSent":
        if request.method == u"PUT":
            response.content = b"PASS: First PUT request."
            setState(b"FirstPUTSent", token)
        else:
            fail(state)
    elif state == b"FirstPUTSent":
        if request.method == u"PUT":
            response.content = b"PASS: Second PUT request. Preflight worked."
        elif request.method == u"OPTIONS":
            response.headers.set(b"Access-Control-Allow-Methods", b"PUT")
            setState(b"FAILSecondOPTIONSSent", token)
        else:
            fail(state)
    elif state == b"FAILSecondOPTIONSSent":
        if request.method == u"PUT":
            fail(b"Second OPTIONS request was sent. Preflight failed.")
        else:
            fail(state)
    else:
        fail(state)
