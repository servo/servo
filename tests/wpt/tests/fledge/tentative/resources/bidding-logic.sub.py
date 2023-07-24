# General bidding logic script. Depending on query parameters, it can
# simulate a variety of network errors, and its generateBid() and
# reportWin() functions can have arbitrary Javascript code injected
# in them. generateBid() will by default return a bid of 9 for the
# first ad.
def main(request, response):
    error = request.GET.first(b"error", None)

    if error == b"close-connection":
        # Close connection without writing anything, to simulate a network
        # error. The write call is needed to avoid writing the default headers.
        response.writer.write("")
        response.close_connection = True
        return

    if error == b"http-error":
        response.status = (404, b"OK")
    else:
        response.status = (200, b"OK")

    if error == b"wrong-content-type":
        response.headers.set(b"Content-Type", b"application/json")
    elif error != b"no-content-type":
        response.headers.set(b"Content-Type", b"application/javascript")

    if error == b"bad-allow-fledge":
        response.headers.set(b"X-Allow-FLEDGE", b"sometimes")
    elif error == b"fledge-not-allowed":
        response.headers.set(b"X-Allow-FLEDGE", b"false")
    elif error != b"no-allow-fledge":
        response.headers.set(b"X-Allow-FLEDGE", b"true")

    body = b''
    if error == b"no-body":
        return body
    if error != b"no-generateBid":
        body += b"""
            function generateBid(interestGroup, auctionSignals, perBuyerSignals,
                                trustedBiddingSignals, browserSignals,
                                directFromSellerSignals) {
              {{GET[generateBid]}};
              return {
                'bid': 9,
                'render': interestGroup.ads[0].renderUrl
              };
            }"""
    bid = request.GET.first(b"bid", None)
    if bid != None:
      body = body.replace(b"9", bid)
    if error != b"no-reportWin":
        body += b"""
            function reportWin(auctionSignals, perBuyerSignals, sellerSignals,
                              browserSignals, directFromSellerSignals) {
              {{GET[reportWin]}};
            }"""
    return body

