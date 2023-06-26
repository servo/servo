import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = [
        (b"Content-Type", b"text/html"),
        (b"Cache-Control", b"no-cache, no-store, must-revalidate")
    ]
    key = request.GET.first(b"key", None)

    # We serialize the key into JSON, so have to decode it first.
    if key is not None:
      key = key.decode('utf-8')

    body = u"""
        <!DOCTYPE html>
        <script src="/portals/resources/stash-utils.sub.js"></script>
        <script>
            var data = %s;
            if (window.opener)
                window.opener.postMessage(data, "*");
            if (window.top != window)
                window.top.postMessage(data, "*");

            const key = %s;
            if (key)
                StashUtils.putValue(key, data);
        </script>
    """ % (json.dumps({
        u"dest": isomorphic_decode(request.headers.get(b"sec-fetch-dest", b"")),
        u"mode": isomorphic_decode(request.headers.get(b"sec-fetch-mode", b"")),
        u"site": isomorphic_decode(request.headers.get(b"sec-fetch-site", b"")),
        u"user": isomorphic_decode(request.headers.get(b"sec-fetch-user", b"")),
        }), json.dumps(key))
    return headers, body
