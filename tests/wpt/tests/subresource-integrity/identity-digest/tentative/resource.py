'''
`Integrity-Digest` helper, generating responses that:

*   Include or exclude an `Integrity-Digest` header depending on the request's
    `digest` parameter.

*   Include or exclude `Access-Control-Allow-Origin: *` depending on the
    request's `cors` parameter.

*   Sets a `Content-Type` header from the request's `type` parameter.

*   Echos the `body` parameter into the response body.
'''
def main(request, response):
  digest = request.GET.first(b'digest', b'')
  if digest:
    response.headers.set(b'identity-digest', digest)

  cors = request.GET.first(b'cors', '')
  if cors:
    response.headers.set(b'access-control-allow-origin', b'*')

  response.headers.set(b'content-type',
                       request.GET.first(b'type', b'text/plain'))

  response.status_code = 200
  response.content = request.GET.first(b'body', '')
