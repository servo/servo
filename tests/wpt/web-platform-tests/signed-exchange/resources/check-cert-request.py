import os

def main(request, response):
    CertChainMimeType = "application/cert-chain+cbor"

    if request.headers.get("Accept") != CertChainMimeType:
        return 400, [], "Bad Request"

    path = os.path.join(os.path.dirname(__file__), "127.0.0.1.sxg.pem.cbor")
    body = open(path, "rb").read()
    return 200, [("Content-Type", CertChainMimeType)], body
