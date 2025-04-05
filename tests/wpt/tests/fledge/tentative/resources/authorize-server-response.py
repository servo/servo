def main(request, response):
  response.status = (200, b"OK")
  response.headers.set(b"Content-Type", b"text/plain")
  if b"hashes" in request.GET:
    hash_list = request.GET.get_list(b"hashes")
    response.headers.set(b"Ad-Auction-Result",
                        b",".join(hash_list))
  if b"nonces" in request.GET:
    nonce_list = request.GET.get_list(b"nonces")
    response.headers.set(b"Ad-Auction-Result-Nonce",
                        b",".join(nonce_list))
