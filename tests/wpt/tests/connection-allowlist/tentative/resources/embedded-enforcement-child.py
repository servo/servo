# Serves the framed document for the Connection-Allowlist embedded-enforcement
# WPTs. Response headers are chosen from query parameters so the parent can
# exercise the opt-in handshake:
#
#   allow_from=<origin|*>  -> Allow-Connection-Allowlist-From: <value>
#   allowlist=<sf-value>   -> Connection-Allowlist: <value>
#
# On load the document messages the top-level frame with the request headers it
# received, so tests can observe both that a navigation committed (i.e. was
# allowed) and which headers the browser emitted. A navigation that embedded
# enforcement blocks commits a cross-origin error document that never runs this
# script and so never sends the message.
import json

from wptserve.utils import isomorphic_decode


def main(request, response):
    headers = [(b"Content-Type", b"text/html")]

    allow_from = request.GET.first(b"allow_from", None)
    if allow_from is not None:
        headers.append((b"Allow-Connection-Allowlist-From", allow_from))

    allowlist = request.GET.first(b"allowlist", None)
    if allowlist is not None:
        headers.append((b"Connection-Allowlist", allowlist))

    def echo(name):
        value = request.headers.get(name)
        return isomorphic_decode(value) if value is not None else None

    # Echo request headers so tests can assert the requirement was delivered as
    # `Sec-Required-Connection-Allowlist` and that no header injection occurred.
    message = {
        u"connectionAllowlistChildLoaded": True,
        u"id": isomorphic_decode(request.GET.first(b"id", b"")),
        u"secRequiredConnectionAllowlist":
            echo(b"Sec-Required-Connection-Allowlist"),
        u"xInjected": echo(b"X-Injected"),
    }
    body = u'''<!DOCTYPE html>
<meta charset="utf-8">
<script>
  window.top.postMessage(%s, "*");
</script>
<body>child</body>
''' % json.dumps(message)
    return headers, body
