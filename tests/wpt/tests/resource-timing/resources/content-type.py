def main(request, response):
    if b'content_type' in request.GET:
      response.headers.set(b'content-type', request.GET.first(b'content_type'))
    if b'allow_origin' in request.GET:
      response.headers.set(b'access-control-allow-origin', request.GET.first(b'allow_origin'))