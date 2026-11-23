'''
SRI Message Signature helper for `accept-signature` header validation for
<script> element requests.

It compares the `accept-signature` header delivered with a request to a
`header` GET parameter. If they match, a `matched` attribute on the current
script element will be set to true.
'''
def main(request, response):
  actual_header = request.headers.get(b'accept-signature', b'')
  expected_header = request.GET.first(b'header', b'')

  # Set common aspects of the response:
  response.status = 200
  response.headers.set(b'content-type', b'application/json')
  response.headers.set(b'access-control-allow-origin', b'*')
  response.headers.set(b'signature-input', \
                       b'signature=("unencoded-digest";sf); '      \
                       b'keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs="; ' \
                       b'tag="ed25519-integrity"')

  # Do the exciting and complicated matching calculation:
  body = b'document.currentScript.setAttribute(`matched`, false);'
  digest = b'es+3YnsBqgi4mkbDZd3Vghz6PsqpNeg5CEJn7WOKzJI='
  signature = b'RLBUQN/+Wydbs24v6AvT/ZKI2PPRGlsOtNOm1OavgGZxTY8q2JqZjTanUNjfIiInXIQExrM5qAtSvnXKazhUAQ=='
  if actual_header == expected_header:
    body = b'document.currentScript.setAttribute(`matched`, true);'
    digest = b'dq6r7uJehA7JvZk7hczA4TM0uQ5Ad9WkKKihnuQ+B3c='
    signature = b'lPat3V34wDgxt1guVhKZDPT6Kk2rRijxfAV5PJ62wObt1GufpX7xzVUuOD5IdlShnUqRvJFyvpDTUu3MLWYfAQ=='

  # Then set those bits.
  response.content = body
  response.headers.set(b'unencoded-digest', b'sha-256=:%s:' % digest)
  response.headers.set(b'signature', b'signature=:%s:' % signature)
