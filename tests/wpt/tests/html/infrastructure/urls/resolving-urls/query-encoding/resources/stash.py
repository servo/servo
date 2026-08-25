import re

def main(request, response):
    key = request.GET[b'id']
    action = request.GET[b'action']
    if action == b'put':
        # We want the raw input for 'q'
        q = re.search(r'q=([^&]+)', request.url_parts.query).groups()[0]
        request.server.stash.put(key, q)
        return [(b"Content-Type", b"text/html")], u'Put %s' % q
    else:
        q = request.server.stash.take(key)
        if q != None:
            return [(b"Content-Type", b"text/html")], q
        return [], u""
