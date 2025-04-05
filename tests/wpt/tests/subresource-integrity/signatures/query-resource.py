'''
SRI Message Signature helper for `@query` tests

These all represent the following response:

> HTTP/1.1 200 OK
> Date: Tue, 20 Apr 2021 02:07:56 GMT
> Content-Type: application/json
> Unencoded-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
> Content-Length: 18
> Signature-Input: signature=("unencoded-digest";sf "@query";req); \
>                  keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";       \
>                  tag="sri"
> Signature: signature=[SEE NOTE BELOW]
>
> {"hello": "world"}

With the `signature` header governed by the query string.
'''
def getSignature(param):
  if param == b'(empty)':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    return b'nddn9dgy3sPXvxoBcV5jxwCgejS3FOIuzXRmQ05V321MnArHDd1BydFm5Na3Q0gUNQFsBJus4+x8+VkTIjlFBA=='
  elif param == b'':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    return b'F4fftMcvtwmghk6n3J86MpYhDT9fPHInd69SXdKb8SB6cWQJMfaKxM0FmO1fJa/pnB2ThO/Sp077+vLURnDEBw=='
  elif param == b'a':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=a
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    return b'QYLmBgierby9E6Q9ZG92jT28+AF73cFHVgBeX/05hqFIt+MG8niYq3G3YWgxPigS7O1i2Vbxu7eQU1JYxht0Dw=='
  elif param == b'/':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=%2F
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    return b'6RY+KehKbGFhE9RItvGuYvs4uOTD7hTE23w5O/oTsvdM6kuU1gD7Y3x3KIjCxTjiFij+8xlFAyYYLsv3LaQSDg=='
  elif param == b'\xc3\xbc':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=%C3%BC
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    return b'TMyMXXE6Pw0wBzeuBpaEOr9RI3WaZLNB7Rhu1euibMqTcp35y7JM3bWZICb7keSGZBvWbidbxrWfWFSsw+J0CA=='
  return param

def main(request, response):
  response.status = 200
  response.content = b'{"hello": "world"}'

  response.headers.set(b'content-type', b'application/json')
  response.headers.set(b'access-control-allow-origin', b'*')
  response.headers.set(b'unencoded-digest', b'sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:')
  response.headers.set(b'signature-input', \
                       b'signature=("unencoded-digest";sf "@query";req); '      \
                       b'keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs="; ' \
                       b'tag="sri"')

  sig = getSignature(request.GET.first(b'test', b'(empty)'))
  response.headers.set(b'signature', b'signature=:%s:' % sig)
