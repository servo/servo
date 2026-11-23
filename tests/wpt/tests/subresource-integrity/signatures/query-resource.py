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
>                  tag="ed25519-integrity"
> Signature: signature=[SEE NOTE BELOW]
>
> {"hello": "world"}

With the `signature` header governed by the query string.
'''
def getSignature(param):
  if param == b'(empty)':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="ed25519-integrity"
    return b'pfVf9zPrIHQn0aQ2mYfvOX11DZNKaOynnXGok74DTLpRzUl2amXk8c44Xcwhg8N/A17boFUL5TArkKD/KansBQ=='
  elif param == b'':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="ed25519-integrity"
    return b'J7Q5G4BSnks4ExWwyll2MNjnQvR4X1k3HIABgLc/17V7xtfS7AaGQgt7/wc6sDiBTcmO/R9yJUU3vjkLntZSDg=='
  elif param == b'a':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=a
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="ed25519-integrity"
    return b'ZAU2N74PTDhp70CYIq1HL3gLUbqP7yk6uu4hVzXBnrgaNlErl02s3+P/rsqsMyJIUy2h6PaYM/6AqOszWTF+Cw=='
  elif param == b'/':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=%2F
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="ed25519-integrity"
    return b'lFKLrekwpdekJQAgE7MDXC38XGUagS6bqc8IEJTnRBdUnxucvnDb7XJRK8zSlno9z+F8SgLcIGfa0BPCd2KkDA=='
  elif param == b'\xc3\xbc':
    # "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    # "@query";req: ?test=%C3%BC
    # "@signature-params": ("unencoded-digest";sf "@query";req);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="ed25519-integrity"
    return b'cBpdDjk0us/5LamB7APkdyNnYI7PxQTWH5W+rpUsZIOCv0EPXO6/o9l/2w832SxPL3KlA3b+SJsmz30QmAZ4Cw=='
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
                       b'tag="ed25519-integrity"')

  sig = getSignature(request.GET.first(b'test', b'(empty)'))
  response.headers.set(b'signature', b'signature=:%s:' % sig)
