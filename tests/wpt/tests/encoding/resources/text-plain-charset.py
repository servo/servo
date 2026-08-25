def main(request, response):
  response.headers.set(b"Content-Type", b"text/plain;charset=" + request.GET.first(b"label"))
  response.content = b"hello encoding"
