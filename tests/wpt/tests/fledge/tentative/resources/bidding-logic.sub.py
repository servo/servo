from pathlib import Path

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
        response.headers.set(b"Ad-Auction-Allowed", b"sometimes")
    elif error == b"fledge-not-allowed":
        response.headers.set(b"Ad-Auction-Allowed", b"false")
    elif error != b"no-allow-fledge":
        response.headers.set(b"Ad-Auction-Allowed", b"true")

    if error == b"no-body":
        return b''

    body = (Path(__file__).parent.resolve() / 'worklet-helpers.js').read_text().encode("ASCII")
    if error != b"no-generateBid":
        # Use bid query param if present. Otherwise, use a bid of 9.
        bid = (request.GET.first(b"bid", None) or b"9").decode("ASCII")

        bidCurrency = ""
        bidCurrencyParam = request.GET.first(b"bidCurrency", None)
        if bidCurrencyParam != None:
            bidCurrency = "bidCurrency: '" + bidCurrencyParam.decode("ASCII") + "',"

        allowComponentAuction = ""
        allowComponentAuctionParam = request.GET.first(b"allowComponentAuction", None)
        if allowComponentAuctionParam != None:
            allowComponentAuction = f"allowComponentAuction: {allowComponentAuctionParam.decode('ASCII')},"

        body += f"""
            function generateBid(interestGroup, auctionSignals, perBuyerSignals,
                                trustedBiddingSignals, browserSignals,
                                directFromSellerSignals) {{
              {{{{GET[generateBid]}}}};
              return {{
                bid: {bid},
                {bidCurrency}
                {allowComponentAuction}
                render: interestGroup.ads[0].renderURL
              }};
            }}""".encode()
    if error != b"no-reportWin":
        body += b"""
            function reportWin(auctionSignals, perBuyerSignals, sellerSignals,
                              browserSignals, directFromSellerSignals) {
              {{GET[reportWin]}};
            }"""
    return body
