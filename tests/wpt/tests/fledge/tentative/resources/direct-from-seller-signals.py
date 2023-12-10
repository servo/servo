import json

# Script to return hardcoded "Ad-Auction-Signals" header to test header-based
# directFromSellerSignals. Requires a "Sec-Ad-Auction-Fetch" header with value
# of b"?1" in the request, otherwise returns a 400 response.
#
# Header "Negative-Test-Option" is used to return some specific hardcoded
# response for some negative test cases.
#
# For all positive test cases, header "Buyer-Origin" is required to be the
# origin in perBuyerSignals, otherwise return 400 response.
def main(request, response):
    # Append CORS headers if needed.
    if b"origin" in request.headers:
      response.headers.set(b"Access-Control-Allow-Origin",
                          request.headers.get(b"origin"))

    if b"credentials" in request.headers:
      response.headers.set(b"Access-Control-Allow-Credentials",
                          request.headers.get(b"credentials"))

    # Handle CORS preflight requests.
    if request.method == u"OPTIONS":
      if not b"Access-Control-Request-Method" in request.headers:
        response.status = (400, b"Bad Request")
        response.headers.set(b"Content-Type", b"text/plain")
        return "Failed to get access-control-request-method in preflight!"

      if not b"Access-Control-Request-Headers" in request.headers:
        response.status = (400, b"Bad Request")
        response.headers.set(b"Content-Type", b"text/plain")
        return "Failed to get access-control-request-headers in preflight!"

      response.headers.set(b"Access-Control-Allow-Methods",
                           request.headers[b"Access-Control-Request-Method"])

      response.headers.set(b"Access-Control-Allow-Headers",
                           request.headers[b"Access-Control-Request-Headers"])

      response.status = (204, b"No Content")
      return

    # Return 400 if there is no "Sec-Ad-Auction-Fetch" header.
    if ("Sec-Ad-Auction-Fetch" not in request.headers or
        request.headers.get("Sec-Ad-Auction-Fetch") != b"?1"):
      response.status = (400, b"Bad Request")
      response.headers.set(b"Content-Type", b"text/plain")
      return "Failed to get Sec-Ad-Auction-Fetch in headers or its value is not \"?1\"."

    # Return 500 to test http error.
    if ("Negative-Test-Option" in request.headers and
        request.headers.get("Negative-Test-Option") == b"HTTP Error"):
      response.status = (500, b"Internal Error")
      response.headers.set(b"Content-Type", b"text/plain")
      return "Test http error with 500 response."

    # Return 200 but without "Ad-Auction-Signals" header.
    if ("Negative-Test-Option" in request.headers and
        request.headers.get("Negative-Test-Option") == b"No Ad-Auction-Signals Header"):
      response.status = (200, b"OK")
      response.headers.set(b"Content-Type", b"text/plain")
      return "Test 200 response without \"Ad-Auction-Signals\" header."

    # Return 200 but with invalid json in "Ad-Auction-Signals" header.
    if ("Negative-Test-Option" in request.headers and
        request.headers.get("Negative-Test-Option") == b"Invalid Json"):
      response.status = (200, b"OK")
      response.headers.set("Ad-Auction-Signals", b"[{\"adSlot\": \"adSlot\", \"sellerSignals\": \"sellerSignals\", \"auctionSignals\":}]")
      response.headers.set(b"Content-Type", b"text/plain")
      return "Test 200 response with invalid json in \"Ad-Auction-Signals\" header."

    # Return 404 but with valid "Ad-Auction-Signals" header to test network error.
    if ("Negative-Test-Option" in request.headers and
        request.headers.get("Negative-Test-Option") == b"Network Error"):
      response.status = (404, b"Not Found")
      adAuctionSignals = json.dumps(
         [{
            "adSlot": "adSlot",
            "sellerSignals": "sellerSignals",
            "auctionSignals": "auctionSignals"
          }])
      response.headers.set("Ad-Auction-Signals", adAuctionSignals)
      response.headers.set(b"Content-Type", b"text/plain")
      return "Test network error with 400 response code and valid \"Ad-Auction-Signals\" header."

    # For positive test cases, buyer-origin is required, otherwise return 400.
    if "Buyer-Origin" not in request.headers:
      response.status = (400, "Bad Request")
      response.headers.set(b"Content-Type", b"text/plain")
      return "Failed to get Buyer-Origin in headers."

    response.status = (200, b"OK")
    buyerOrigin = request.headers.get("Buyer-Origin").decode('utf-8')

    altResponse = request.headers.get("Alternative-Response")

    if altResponse == b"Overwrite adSlot/1":
      adAuctionSignals = json.dumps(
        [{
          "adSlot": "adSlot/1",
          "sellerSignals": "altSellerSignals/1",
        }])
    elif altResponse == b"Overwrite adSlot/1 v2":
      adAuctionSignals = json.dumps(
        [{
          "adSlot": "adSlot/1",
          "sellerSignals": "altV2SellerSignals/1",
        }])
    elif altResponse == b"Two keys with same values":
      adAuctionSignals = json.dumps(
        [{
          "adSlot": "adSlot/1",
          "sellerSignals": "sameSellerSignals",
          "auctionSignals": "sameAuctionSignals",
          "perBuyerSignals": { buyerOrigin: "samePerBuyerSignals" }
        },
        {
          "adSlot": "adSlot/2",
          "sellerSignals": "sameSellerSignals",
          "auctionSignals": "sameAuctionSignals",
          "perBuyerSignals": { buyerOrigin: "samePerBuyerSignals" }
        }])
    elif altResponse == b"Duplicate adSlot/1":
      adAuctionSignals = json.dumps(
        [{
          "adSlot": "adSlot/1",
          "sellerSignals": "firstSellerSignals/1",
        },
        {
          "adSlot": "adSlot/2",
          "sellerSignals": "nonDupSellerSignals/2",
        },
        {
          "adSlot": "adSlot/1",
          "sellerSignals": "secondSellerSignals/1",
        }])
    else:
      adAuctionSignals = json.dumps(
        [{
          "adSlot": "adSlot/0",
        },
        {
          "adSlot": "adSlot/1",
          "sellerSignals": "sellerSignals/1",
        },
        {
          "adSlot": "adSlot/2",
          "auctionSignals": "auctionSignals/2",
        },
        {
          "adSlot": "adSlot/3",
          "perBuyerSignals": { buyerOrigin: "perBuyerSignals/3" }
        },
        {
          "adSlot": "adSlot/4",
          "sellerSignals": "sellerSignals/4",
          "auctionSignals": "auctionSignals/4",
          "perBuyerSignals": { buyerOrigin: "perBuyerSignals/4" }
        },
        {
          "adSlot": "adSlot/5",
          "sellerSignals": "sellerSignals/5",
          "auctionSignals": "auctionSignals/5",
          "perBuyerSignals": { "mismatchOrigin": "perBuyerSignals/5" }
        }])

    response.headers.set("Ad-Auction-Signals", adAuctionSignals)
    response.headers.set(b"Content-Type", b"text/plain")
    return
