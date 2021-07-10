from wptserve.utils import isomorphic_encode

def main(request, response):
    response.status = 302
    response.headers.set(b"Location", isomorphic_encode(request.url[request.url.find(u'?')+1:]))
