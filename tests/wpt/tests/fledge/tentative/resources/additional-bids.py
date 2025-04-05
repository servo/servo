"""Endpoint to return signed additional bids in the appropriate response header.

Additional bids are returned using the "Ad-Auction-Additional-Bid" response
header, as described at
https://github.com/WICG/turtledove/blob/main/FLEDGE.md#63-http-response-headers.

This script generates an "Ad-Auction-Additional-Bid" response header for each of
the pre-formatted additional bid header values provided in a JSON list-valued
`additionalBidHeaderValues` query parameter.

All requests to this endpoint requires a "Sec-Ad-Auction-Fetch" request header
with a value of b"?1"; this entrypoint otherwise returns a 400 response.
"""

import json

import fledge.tentative.resources.fledge_http_server_util as fledge_http_server_util


class BadRequestError(Exception):
  pass


def main(request, response):
  try:
    if fledge_http_server_util.handle_cors_headers_fail_if_preflight(request, response):
      return

    # Verify that Sec-Ad-Auction-Fetch is present
    if request.headers.get("Sec-Ad-Auction-Fetch", default=b"").decode("utf-8") != "?1":
      raise BadRequestError("Sec-Ad-Auction-Fetch missing or unexpected value; expected '?1'")

    # Return each additional bid in its own header
    additional_bid_header_values = request.GET.get(b"additionalBidHeaderValues", default=b"").decode("utf-8")
    if not additional_bid_header_values:
      raise BadRequestError("Missing 'additionalBidHeaderValues' parameter")
    for additional_bid_header_value in json.loads(additional_bid_header_values):
      response.headers.append(
          b"Ad-Auction-Additional-Bid", additional_bid_header_value.encode("utf-8"))

    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"text/plain")

  except BadRequestError as error:
    response.status = (400, b"Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    response.content = str(error)

  except Exception as exception:
    response.status = (500, b"Internal Server Error")
    response.content = str(exception)
