from wptserve.utils import isomorphic_decode

def main(request, response):
    id = request.GET[b'id']
    encoding = request.GET[b'encoding']
    mode = request.GET[b'mode']
    iframe = u""
    if mode == b'NETWORK':
        iframe = u"<iframe src='stash.py?q=%%C3%%A5&id=%s&action=put'></iframe>" % isomorphic_decode(id)
    doc = u"""<!doctype html>
<html manifest="manifest.py?id=%s&encoding=%s&mode=%s">
%s
""" % (isomorphic_decode(id), isomorphic_decode(encoding), isomorphic_decode(mode), iframe)
    return [(b"Content-Type", b"text/html; charset=%s" % encoding)], doc.encode(isomorphic_decode(encoding))
