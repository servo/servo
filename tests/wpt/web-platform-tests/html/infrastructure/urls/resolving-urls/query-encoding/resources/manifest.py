def main(request, response):
    id = request.GET['id']
    mode = request.GET['mode']
    fallback_url = ""
    if mode == "FALLBACK":
        fallback_url = "fallback-namespace/"
    manifest = u"""CACHE MANIFEST

%s:
%s stash.py?q=\u00E5&id=%s&action=put
""" % (mode, fallback_url, id)
    return [("Content-Type", "text/cache-manifest; charset=%s" % request.GET['encoding'])], manifest.encode('utf-8') # charset should be ignored for cache manifests
