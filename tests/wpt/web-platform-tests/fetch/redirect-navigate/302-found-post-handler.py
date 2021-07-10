from wptserve.utils import isomorphic_encode

def main(request, response):
    if request.method == u"POST":
        response.add_required_headers = False
        response.writer.write_status(302)
        response.writer.write_header(b"Location", isomorphic_encode(request.url))
        response.writer.end_headers()
        response.writer.write(b"")
    elif request.method == u"GET":
        return ([(b"Content-Type", b"text/plain")],
                b"OK")
    else:
        return ([(b"Content-Type", b"text/plain")],
                b"FAIL")