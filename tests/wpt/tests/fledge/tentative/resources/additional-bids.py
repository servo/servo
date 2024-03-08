"""Endpoint to return additional bids in the appropriate response header.

Additional bids are returned using the "Ad-Auction-Additional-Bid" response
header, as described at
https://github.com/WICG/turtledove/blob/main/FLEDGE.md#63-http-response-headers.

This script generates one of "Ad-Auction-Additional-Bid" response header for
each additional bid provided in a url-encoded `additionalBids` query parameter.

All requests to this endpoint requires a "Sec-Ad-Auction-Fetch" request header
with a value of b"?1"; this entrypoint otherwise returns a 400 response.
"""
import json
import base64

import fledge.tentative.resources.fledge_http_server_util as fledge_http_server_util


class BadRequestError(Exception):
  pass


def main(request, response):
  try:
    if fledge_http_server_util.handle_cors_headers_and_preflight(request, response):
      return

    # Verify that Sec-Ad-Auction-Fetch is present
    if (request.headers.get("Sec-Ad-Auction-Fetch", default=b"").decode("utf-8") != "?1"):
      raise BadRequestError("Sec-Ad-Auction-Fetch missing or unexpected value; expected '?1'")

    # Return each signed additional bid in its own header
    additional_bids = request.GET.get(b"additionalBids", default=b"").decode("utf-8")
    if not additional_bids:
      raise BadRequestError("Missing 'additionalBids' parameter")
    for additional_bid in json.loads(additional_bids):
      additional_bid_string = json.dumps(additional_bid)
      auction_nonce = additional_bid.get("auctionNonce", None)
      if not auction_nonce:
        raise BadRequestError("Additional bid missing required 'auctionNonce' field")
      signed_additional_bid = json.dumps({
        "bid": additional_bid_string,
        "signatures": []
      })
      additional_bid_header_value = (auction_nonce.encode("utf-8") + b":" +
                                     base64.b64encode(signed_additional_bid.encode("utf-8")))
      response.headers.append(b"Ad-Auction-Additional-Bid", additional_bid_header_value)

    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"text/plain")

  except BadRequestError as error:
    response.status = (400, b"Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    response.content = str(error)

  except Exception as exception:
    response.status = (500, b"Internal Server Error")
    response.content = str(exception)
