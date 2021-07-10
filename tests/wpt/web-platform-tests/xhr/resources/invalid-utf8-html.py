from six import int2byte

def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=utf-8")]
    content = int2byte(0xff)

    return headers, content
