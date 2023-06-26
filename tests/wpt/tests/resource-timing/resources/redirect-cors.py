def main(request, response):
    if b"allow_origin" in request.GET:
      response.headers.set(b"Access-Control-Allow-Origin",
        request.GET.first(b"allow_origin"))

    if b"timing_allow_origin" in request.GET:
        response.headers.set(b"Timing-Allow-Origin",
          request.GET.first(b"timing_allow_origin"))

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
      location = request.GET.first(b"location")
      response.status = 302
      response.headers.set(b"Location", location)
