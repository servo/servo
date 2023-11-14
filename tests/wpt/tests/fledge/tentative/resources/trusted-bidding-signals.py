import json
from urllib.parse import unquote_plus
from fledge.tentative.resources.fledge_http_server_util import headersToAscii

# Script to generate trusted bidding signals. The responses depends on the
# keys and interestGroupNames - some result in entire response failures, others
# affect only their own value. Keys are preferentially used over
# interestGroupName, since keys are composible, but some tests need to cover
# there being no keys.
def main(request, response):
    hostname = None
    keys = None
    interestGroupNames = None

    # Manually parse query params. Can't use request.GET because it unescapes as well as splitting,
    # and commas mean very different things from escaped commas.
    for param in request.url_parts.query.split("&"):
        pair = param.split("=", 1)
        if len(pair) != 2:
            return fail(response, "Bad query parameter: " + param)
        # Browsers should escape query params consistently.
        if "%20" in pair[1]:
            return fail(response, "Query parameter should escape using '+': " + param)

        # Hostname can't be empty. The empty string can be a key or interest group name, though.
        if pair[0] == "hostname" and hostname == None and len(pair[1]) > 0:
            hostname = pair[1]
            continue
        if pair[0] == "keys" and keys == None:
            keys = list(map(unquote_plus, pair[1].split(",")))
            continue
        if pair[0] == "interestGroupNames" and interestGroupNames == None:
            interestGroupNames = list(map(unquote_plus, pair[1].split(",")))
            continue
        return fail(response, "Unexpected query parameter: " + param)

    # "interestGroupNames" and "hostname" are mandatory.
    if not hostname:
        return fail(response, "hostname missing")
    if not interestGroupNames:
        return fail(response, "interestGroupNames missing")

    response.status = (200, b"OK")

    # The JSON representation of this is used as the response body. This does
    # not currently include a "perInterestGroupData" object.
    responseBody = {"keys": {}}

    # Set when certain special keys are observed, used in place of the JSON
    # representation of `responseBody`, when set.
    body = None

    contentType = "application/json"
    adAuctionAllowed = "true"
    dataVersion = None
    if keys:
        for key in keys:
            value = "default value"
            if key == "close-connection":
                # Close connection without writing anything, to simulate a
                # network error. The write call is needed to avoid writing the
                # default headers.
                response.writer.write("")
                response.close_connection = True
                return
            elif key.startswith("replace-body:"):
                # Replace entire response body. Continue to run through other
                # keys, to allow them to modify request headers.
                body = key.split(':', 1)[1]
            elif key.startswith("data-version:"):
                dataVersion = key.split(':', 1)[1]
            elif key == "http-error":
                response.status = (404, b"Not found")
            elif key == "no-content-type":
                contentType = None
            elif key == "wrong-content-type":
                contentType = 'text/plain'
            elif key == "bad-ad-auction-allowed":
                adAuctionAllowed = "sometimes"
            elif key == "ad-auction-not-allowed":
                adAuctionAllowed = "false"
            elif key == "no-ad-auction-allow":
                adAuctionAllowed = None
            elif key == "no-value":
                continue
            elif key == "wrong-value":
                responseBody["keys"]["another-value"] = "another-value"
                continue
            elif key == "null-value":
                value = None
            elif key == "num-value":
                value = 1
            elif key == "string-value":
                value = "1"
            elif key == "array-value":
                value = [1, "foo", None]
            elif key == "object-value":
                value = {"a":"b", "c":["d"]}
            elif key == "interest-group-names":
                value = json.dumps(interestGroupNames)
            elif key == "hostname":
                value = request.GET.first(b"hostname", b"not-found").decode("ASCII")
            elif key == "headers":
                value = headersToAscii(request.headers)
            responseBody["keys"][key] = value

    if "data-version" in interestGroupNames:
        dataVersion = "4"

    if contentType:
        response.headers.set("Content-Type", contentType)
    if adAuctionAllowed:
        response.headers.set("Ad-Auction-Allowed", adAuctionAllowed)
    if dataVersion:
        response.headers.set("Data-Version", dataVersion)
    response.headers.set("Ad-Auction-Bidding-Signals-Format-Version", "2")

    if body != None:
        return body
    return json.dumps(responseBody)

def fail(response, body):
    response.status = (400, "Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    return body
