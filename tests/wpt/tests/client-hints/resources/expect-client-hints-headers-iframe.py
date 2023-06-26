from wptserve.utils import isomorphic_decode

import importlib
client_hints_list = importlib.import_module("client-hints.resources.clienthintslist").client_hints_list

def main(request, response):
    """
    Simple handler that returns an HTML response that passes when the required
    Client Hints are received as request headers.
    """

    result = u"PASS"
    log = u""
    for value in client_hints_list():
        should = (request.GET[value.lower()] == b"true")
        present = request.headers.get(value.lower()) or request.headers.get(value)
        if present:
            log += isomorphic_decode(value) + u" " + str(should) + u" " + isomorphic_decode(present) + u", "
        else:
            log += isomorphic_decode(value) + u" " + str(should) + u" " + str(present) + u", "
        if (should and not present) or (not should and present):
            if present:
                result = u"FAIL " + isomorphic_decode(value) + u" " + str(should) + u" " + isomorphic_decode(present)
            else:
                result = u"FAIL " + isomorphic_decode(value) + u" " + str(should) + u" " + str(present)
            break

    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    body = u"<script>console.log('" + log + u"'); window.parent.postMessage('" + result + u"', '*');</script>"

    response.content = body
