# This will echo the 'Sec-Required-Document-Policy' request header in the body
# of the response, as well as in the 'Document-Policy' response header (to
# ensure the response is loaded by a user agent which is implementing document
# policy.)
import json

def main(request, response):
    msg = {}
    headers = [('Content-Type', 'text/html')]

    srdp = request.headers.get('Sec-Required-Document-Policy')
    if srdp:
      msg['requiredPolicy'] = srdp
      headers.append(('Document-Policy', srdp))

    frameId = request.GET.first('id',None)
    if frameId:
      msg['id'] = frameId

    content = """<!DOCTYPE html>
<script>
top.postMessage(%s, "*");
</script>
%s
""" % (json.dumps(msg), srdp)

    return (200, 'OK'), headers, content

