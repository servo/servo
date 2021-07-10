from six import int2byte

def main(request, response):
    headers = [(b"Content-type", b"application/xml;charset=windows-1252")]
    content = b'<' + int2byte(0xff) + b'/>'

    return headers, content
