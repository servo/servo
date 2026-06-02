import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    testinput = request.POST.first(b"testinput")
    value = isomorphic_decode(testinput.value)
    body = u"<script>parent.postMessage(" + json.dumps(value) + u", '*');</script>"
    return headers, body
