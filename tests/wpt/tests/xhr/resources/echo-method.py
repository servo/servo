# This handler is designed to verify that UAs correctly discard the body of
# responses to HTTP HEAD requests. If the response body is written to a
# separate TCP packet, then this behavior cannot be verified. This handler uses
# the response writer to ensure that the body is transmitted in the same packet
# as the headers. In this way, non-conforming UAs will consistently fail the
# associated test.

def main(request, response):
    content = request.method

    response.add_required_headers = False
    response.writer.write(u'''HTTP/1.1 200 OK
Content-type: text/plain
Content-Length: {}

{}'''.format(len(content), content))
