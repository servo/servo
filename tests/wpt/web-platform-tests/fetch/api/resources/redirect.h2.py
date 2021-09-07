from wptserve.utils import isomorphic_decode, isomorphic_encode

def handle_headers(frame, request, response):
    status = 302
    if b'redirect_status' in request.GET:
        status = int(request.GET[b'redirect_status'])
    response.status = status

    if b'location' in request.GET:
      url = isomorphic_decode(request.GET[b'location'])
      response.headers[b'Location'] = isomorphic_encode(url)

    response.headers.update([('Content-Type', 'text/plain')])
    response.write_status_headers()
