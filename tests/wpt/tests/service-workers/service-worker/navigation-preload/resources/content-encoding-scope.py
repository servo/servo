import gzip

def main(request, response):
    response.headers.set(b"Content-Type", b"text/html; charset=UTF-8")
    response.headers.set(b"Content-Encoding", b"gzip")
    response.content = gzip.compress(bytes(u"Hello World", 'utf-8'))

