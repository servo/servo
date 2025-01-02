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

import fledge.tentative.resources.ed25519 as ed25519
import fledge.tentative.resources.fledge_http_server_util as fledge_http_server_util


class BadRequestError(Exception):
  pass


def _generate_signature(message, base64_encoded_secret_key):
  """Returns a signature entry for a signed additional bid.

  Args:
    base64_encoded_secret_key: base64-encoded Ed25519 key with which to sign
        the message. From this secret key, the public key can be deduced, which
        becomes part of the signature entry.
    message: The additional bid text (or other text if generating an invalid
        signature) to sign.
  """
  secret_key = base64.b64decode(base64_encoded_secret_key.encode("utf-8"))
  public_key = ed25519.publickey_unsafe(secret_key)
  signature = ed25519.signature_unsafe(
      message.encode("utf-8"), secret_key, public_key)
  return {
      "key": base64.b64encode(public_key).decode("utf-8"),
      "signature": base64.b64encode(signature).decode("utf-8")
  }


def _sign_additional_bid(additional_bid_string,
                         secret_keys_for_valid_signatures,
                         secret_keys_for_invalid_signatures):
  """Returns a signed additional bid given an additional bid and secret keys.

  Args:
    additional_bid_string: string representation of the additional bid
    secret_keys_for_valid_signatures: a list of strings, each a base64-encoded
        Ed25519 secret key with which to sign the additional bid
    secret_keys_for_invalid_signatures: a list of strings, each a base64-encoded
        Ed25519 secret key with which to incorrectly sign the additional bid
  """
  signatures = []
  signatures.extend(
      _generate_signature(additional_bid_string, secret_key)
      for secret_key in secret_keys_for_valid_signatures)

  # For invalid signatures, we use the correct secret key to sign a different
  # message - the additional bid prepended by 'invalid' - so that the signature
  # is a structually valid signature but can't be used to verify the additional
  # bid.
  signatures.extend(
      _generate_signature("invalid" + additional_bid_string, secret_key)
       for secret_key in secret_keys_for_invalid_signatures)

  return json.dumps({
    "bid": additional_bid_string,
    "signatures": signatures
  })


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
      # Each additional bid may have associated testMetadata. Remove this from
      # the additional bid and use it to adjust the behavior of this handler.
      test_metadata = additional_bid.pop("testMetadata", {})
      seller_nonce = test_metadata.get("sellerNonce", None)
      remove_auction_nonce_from_bid = test_metadata.get("removeAuctionNonceFromBid", False)
      bid_auction_nonce_override = test_metadata.get("bidAuctionNonceOverride", None)
      if remove_auction_nonce_from_bid:
        auction_nonce = additional_bid.pop("auctionNonce", None)
      else:
        auction_nonce = additional_bid.get("auctionNonce", None)
      if bid_auction_nonce_override:
        additional_bid["auctionNonce"] = bid_auction_nonce_override
      if not auction_nonce:
        raise BadRequestError("Additional bid missing required 'auctionNonce' field")
      signed_additional_bid = _sign_additional_bid(
          json.dumps(additional_bid),
          test_metadata.get("secretKeysForValidSignatures", []),
          test_metadata.get("secretKeysForInvalidSignatures", []))
      if seller_nonce:
        additional_bid_header_value = (auction_nonce.encode("utf-8") + b":" +
                                       seller_nonce.encode("utf-8") + b":" +
                                       base64.b64encode(signed_additional_bid.encode("utf-8")))
      else:
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
