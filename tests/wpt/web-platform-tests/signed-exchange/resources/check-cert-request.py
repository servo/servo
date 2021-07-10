import os

from wptserve.utils import isomorphic_decode

def main(request, response):
    CertChainMimeType = b"application/cert-chain+cbor"

    if request.headers.get(b"Accept") != CertChainMimeType:
        return 400, [], u"Bad Request"

    path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"127.0.0.1.sxg.pem.cbor")
    body = open(path, u"rb").read()
    return 200, [(b"Content-Type", CertChainMimeType)], body
