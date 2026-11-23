def main(request, response):
    if b'status' in request.GET:
      response.status = int(request.GET.first(b'status'))
    if b'content_type' in request.GET:
      response.headers.set(b'content-type', request.GET.first(b'content_type'))
    if b'allow_origin' in request.GET:
      response.headers.set(b'access-control-allow-origin', request.GET.first(b'allow_origin'))
    if b'tao_value' in request.GET:
      response.headers.set(b'timing-allow-origin', request.GET.first(b'tao_value'))
