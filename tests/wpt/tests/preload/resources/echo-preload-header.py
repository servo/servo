import os
from wptserve.utils import isomorphic_encode

def main(request, response):
    response.headers.set(b"Content-Type", request.GET.first(b"type"))
    link = request.GET.first(b"link")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Access-Control-Allow-Credentials", b"true")
    if link is not None:
        response.headers.set(b"Link", link)

    if b"file" in request.GET:
        path = os.path.join(os.path.dirname(isomorphic_encode(__file__)), request.GET.first(b"file"));
        response.content = open(path, mode=u'rb').read();
    else:
        return request.GET.first(b"content")
