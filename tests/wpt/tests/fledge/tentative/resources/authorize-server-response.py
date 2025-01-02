def main(request, response):
  response.status = (200, b"OK")
  response.headers.set(b"Content-Type", b"text/plain")
  hash_list = request.GET.get_list(b"hashes")
  response.headers.set(b"Ad-Auction-Result",
                       b",".join(hash_list))
