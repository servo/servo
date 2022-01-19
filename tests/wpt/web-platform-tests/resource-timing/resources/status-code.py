def main(request, response):
    status = request.GET.first(b'status')
    response.status = (status, b"");
    if b'tao_value' in request.GET:
      response.headers.set(b'timing-allow-origin', request.GET.first(b'tao_value'))

