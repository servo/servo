from wptserve.utils import isomorphic_encode

def main(request, response):
    if request.method == u"OPTIONS":
        response.headers.set(b"Content-Type", b"text/plain")
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")
        response.headers.set(b"Access-Control-Allow-Methods", b"PUT")
        response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))

    elif request.method == u"PUT":
        response.headers.set(b"Content-Type", b"text/plain")
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")
        response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
        response.content = b"PASS: Cross-domain access allowed."
        try:
            response.content += b"\n" + request.body
        except:
            response.content += b"Could not read in content."

    else:
        response.headers.set(b"Content-Type", b"text/plain")
        response.content = b"Wrong method: " + isomorphic_encode(request.method)
