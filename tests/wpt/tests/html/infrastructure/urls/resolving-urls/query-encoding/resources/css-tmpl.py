from wptserve.utils import isomorphic_decode

def main(request, response):
    encoding = request.GET[b'encoding']
    tmpl = request.GET[b'tmpl']
    sheet = isomorphic_decode(tmpl) % u'\\0000E5'
    return [(b"Content-Type", b"text/css; charset=%s" % encoding)], sheet.encode(isomorphic_decode(encoding))
