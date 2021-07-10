# This will echo the 'Sec-Required-Document-Policy' request header in the body
# of the response, as well as in the 'Document-Policy' response header (to
# ensure the response is loaded by a user agent which is implementing document
# policy.)
import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    msg = {}
    headers = [(b'Content-Type', b'text/html')]

    srdp = request.headers.get(b'Sec-Required-Document-Policy')
    if srdp:
      msg[u'requiredPolicy'] = isomorphic_decode(srdp)
      headers.append((b'Document-Policy', srdp))

    frameId = request.GET.first(b'id',None)
    if frameId:
      msg[u'id'] = isomorphic_decode(frameId)

    content = u"""<!DOCTYPE html>
<script>
top.postMessage(%s, "*");
</script>
%s
""" % (json.dumps(msg), isomorphic_decode(srdp) if srdp != None else srdp)

    return (200, u'OK'), headers, content

