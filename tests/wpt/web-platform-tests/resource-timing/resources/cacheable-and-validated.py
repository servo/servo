def main(request, response):
    # Headers need to be set before `response.writer` writes out the response.
    tao = request.GET.get(b'timing_allow_origin')
    if tao:
      response.headers.set(b"Timing-Allow-Origin", tao)

    if b'origin' in request.headers:
      origin = request.headers[b'origin']
      response.headers.set(b'Access-Control-Allow-Origin', origin)

    content = request.GET.first(b'content')
    response.headers.set(b'Cache-Control', b'max-age=60')
    response.headers.set(b'ETag', b'assdfsdfe')

    # Handle CORS-preflights of non-simple requests.
    if request.method == 'OPTIONS':
      response.status = 204
      requested_method = request.headers.get(b"Access-Control-Request-Method")
      if requested_method:
        response.headers.set(b"Access-Control-Allow-Methods", requested_method)
      requested_headers = request.headers.get(b"Access-Control-Request-Headers")
      if requested_headers:
        response.headers.set(b"Access-Control-Allow-Headers", requested_headers)
    else:
      if 'Cache-Control' in request.headers:
        response.status = (304, b'NotModified')
      else:
        response.status = (200, b'OK')
        response.write_status_headers()
        response.writer.write(content)
