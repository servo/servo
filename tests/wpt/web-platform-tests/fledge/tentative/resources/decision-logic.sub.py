# General decision logic script. Depending on query parameters, it can
# simulate a variety of network errors, and its scoreAd() and
# reportResult() functions can have arbitrary Javascript code injected
# in them. scoreAd() will by default return a desirability score of
# twice the bid for each ad, as long as the ad URL ends with the uuid.
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
    if error != b"no-scoreAd":
        body += b"""
            function scoreAd(adMetadata, bid, auctionConfig, trustedScoringSignals,
                            browserSignals) {
              // Don't bid on interest group with the wrong uuid. This is to prevent
              // left over interest groups from other tests from affecting auction
              // results.
              if (!browserSignals.renderUrl.endsWith('{{GET[uuid]}}'))
                return 0;

              {{GET[scoreAd]}};
              return {desirability: 2 * bid, allowComponentAuction: true};
            }"""
    if error != b"no-reportResult":
        body += b"""
            function reportResult(auctionConfig, browserSignals, directFromSellerSignals) {
              {{GET[reportResult]}};
            }"""
    return body
