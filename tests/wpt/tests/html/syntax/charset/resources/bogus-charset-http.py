def main(request, response):
  response.headers.set(b"Content-Type", b"text/html;charset=this-is-not-a-charset")
  # ¢
  response.content = b"\xA2\n"
