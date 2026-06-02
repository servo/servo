import os.path

# Return arbitory content-encoding header value.

def read(file):
    path = os.path.join(os.path.dirname(__file__), file)
    return open(path, u"rb").read()

def main(request, response):
    response.headers.set(b"Content-Type", b"text/javascript")
    response.content = read(u"./dummy.js")

    if b'content_encoding' in request.GET:
        content_encoding = request.GET.first(b"content_encoding")
        response.headers.set(b"Content-Encoding", content_encoding)
