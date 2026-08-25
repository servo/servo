def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=windows-1252")]
    content = chr(0xff)

    return headers, content
