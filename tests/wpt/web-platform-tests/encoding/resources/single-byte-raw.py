from six import int2byte

def main(request, response):
  response.headers.set(b"Content-Type", b"text/plain;charset=" + request.GET.first(b"label"))
  response.content = b"".join(int2byte(byte) for byte in range(255))
