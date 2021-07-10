from wptserve.utils import isomorphic_decode

def main(request, response):
    id = request.GET[b'id']
    mode = request.GET[b'mode']
    fallback_url = u""
    if mode == b"FALLBACK":
        fallback_url = u"fallback-namespace/"
    manifest = u"""CACHE MANIFEST

%s:
%s stash.py?q=\u00E5&id=%s&action=put
""" % (isomorphic_decode(mode), fallback_url, isomorphic_decode(id))
    return [(b"Content-Type", b"text/cache-manifest; charset=%s" % request.GET[b'encoding'])], manifest.encode('utf-8') # charset should be ignored for cache manifests
