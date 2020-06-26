def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=utf-8")]
    content = chr(0xff)

    return headers, content
