'''
SRI Message Signature helper, generating responses that:

*   Include or exclude an `Integrity-Digest` header depending on the request's
    `digest` parameter.
*   Include or exclude an `Signature` header depending on the request's
    `signature` parameter.
*   Include or exclude an `Signature-Input` header depending on the request's
    `signatureInput` parameter.

*   Include or exclude `Access-Control-Allow-Origin: *` depending on the
    request's `cors` parameter.

*   Sets a `Content-Type` header from the request's `type` parameter.

*   Echos the `body` parameter into the response body.
'''
def main(request, response):
  digest = request.GET.first(b'digest', b'')
  signature = request.GET.first(b'signature', b'')
  signatureInput = request.GET.first(b'signatureInput', b'')
  if digest:
    response.headers.set(b'identity-digest', digest)
  if signature:
    response.headers.set(b'signature', signature)
  if signatureInput:
    response.headers.set(b'signature-input', signatureInput)


  cors = request.GET.first(b'cors', '')
  if cors:
    response.headers.set(b'access-control-allow-origin', b'*')

  response.headers.set(b'content-type',
                       request.GET.first(b'type', b'text/plain'))

  # Reflect the `accept-signature` header from the request to the response.
  acceptSigs = request.headers.get(b'accept-signature', b'')
  response.headers.set(b'accept-signature', acceptSigs)
  response.headers.set(b'access-control-expose-headers', b'accept-signature')

  response.status_code = 200
  response.content = request.GET.first(b'body', '')
