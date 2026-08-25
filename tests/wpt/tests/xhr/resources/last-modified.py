from wptserve.utils import isomorphic_decode, isomorphic_encode

def main(request, response):
    import datetime, os
    srcpath = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"well-formed.xml")
    srcmoddt = datetime.datetime.utcfromtimestamp(os.path.getmtime(srcpath))
    response.headers.set(b"Last-Modified", isomorphic_encode(srcmoddt.strftime(u"%a, %d %b %Y %H:%M:%S GMT")))
    response.headers.set(b"Content-Type", b"application/xml")
    return open(srcpath, u"r").read()
