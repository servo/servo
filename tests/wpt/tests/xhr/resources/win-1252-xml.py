def main(request, response):
    headers = [(b"Content-type", b"application/xml;charset=windows-1252")]
    content = b'<\xff/>'

    return headers, content
