import json
from urllib.parse import unquote_plus, urlparse
from fledge.tentative.resources.fledge_http_server_util import headersToAscii

# Script to generate trusted scoring signals. The responses depends on the
# query strings in the ads Urls - some result in entire response failures,
# others affect only their own value. Each renderUrl potentially has a
# signalsParam, which is a comma-delimited list of instructions that can
# each affect either the value associated with the renderUrl, or the
# response as a whole.
def main(request, response):
    hostname = None
    renderUrls = None
    adComponentRenderURLs = None
    # List of {type: <render URL type>, urls: <render URL list>} pairs, where <render URL type> is
    # one of the two render URL dictionary keys used in the response ("renderUrls" or
    # "adComponentRenderURLs"). May be of length 1 or 2, depending on whether there
    # are any component URLs.
    urlLists = []

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
        if pair[0] == "renderUrls" and renderUrls == None:
            renderUrls = list(map(unquote_plus, pair[1].split(",")))
            urlLists.append({"type":"renderUrls", "urls":renderUrls})
            continue
        if pair[0] == "adComponentRenderUrls" and adComponentRenderURLs == None:
            adComponentRenderURLs = list(map(unquote_plus, pair[1].split(",")))
            urlLists.append({"type":"adComponentRenderURLs", "urls":adComponentRenderURLs})
            continue
        return fail(response, "Unexpected query parameter: " + param)

    # "hostname" and "renderUrls" are mandatory.
    if not hostname:
        return fail(response, "hostname missing")
    if not renderUrls:
        return fail(response, "renderUrls missing")

    response.status = (200, b"OK")

    # The JSON representation of this is used as the response body.
    responseBody = {"renderUrls": {}}

    # Set when certain special keys are observed, used in place of the JSON
    # representation of `responseBody`, when set.
    body = None

    contentType = "application/json"
    adAuctionAllowed = "true"
    dataVersion = None
    for urlList in urlLists:
        for renderUrl in urlList["urls"]:
            value = "default value"
            addValue = True

            signalsParams = None
            for param in urlparse(renderUrl).query.split("&"):
                pair = param.split("=", 1)
                if len(pair) != 2:
                    continue
                if pair[0] == "signalsParams":
                    if signalsParams != None:
                        return fail(response, "renderUrl has multiple signalsParams: " + renderUrl)
                    signalsParams = pair[1]
            if signalsParams != None:
                signalsParams = unquote_plus(signalsParams)
                for signalsParam in signalsParams.split(","):
                    if signalsParam == "close-connection":
                        # Close connection without writing anything, to simulate a
                        # network error. The write call is needed to avoid writing the
                        # default headers.
                        response.writer.write("")
                        response.close_connection = True
                        return
                    elif signalsParam.startswith("replace-body:"):
                        # Replace entire response body. Continue to run through other
                        # renderUrls, to allow them to modify request headers.
                        body = signalsParam.split(':', 1)[1]
                    elif signalsParam.startswith("data-version:"):
                        dataVersion = signalsParam.split(':', 1)[1]
                    elif signalsParam == "http-error":
                        response.status = (404, b"Not found")
                    elif signalsParam == "no-content-type":
                        contentType = None
                    elif signalsParam == "wrong-content-type":
                        contentType = 'text/plain'
                    elif signalsParam == "bad-ad-auction-allowed":
                        adAuctionAllowed = "sometimes"
                    elif signalsParam == "ad-auction-not-allowed":
                        adAuctionAllowed = "false"
                    elif signalsParam == "no-ad-auction-allow":
                        adAuctionAllowed = None
                    elif signalsParam == "wrong-url":
                        renderUrl = "https://wrong-url.test/"
                    elif signalsParam == "no-value":
                        addValue = False
                    elif signalsParam == "null-value":
                        value = None
                    elif signalsParam == "num-value":
                        value = 1
                    elif signalsParam == "string-value":
                        value = "1"
                    elif signalsParam == "array-value":
                        value = [1, "foo", None]
                    elif signalsParam == "object-value":
                        value = {"a":"b", "c":["d"]}
                    elif signalsParam == "hostname":
                        value = request.GET.first(b"hostname", b"not-found").decode("ASCII")
                    elif signalsParam == "headers":
                        value = headersToAscii(request.headers)
            if addValue:
                if urlList["type"] not in responseBody:
                    responseBody[urlList["type"]] = {}
                responseBody[urlList["type"]][renderUrl] = value

    if contentType:
        response.headers.set("Content-Type", contentType)
    if adAuctionAllowed:
        response.headers.set("Ad-Auction-Allowed", adAuctionAllowed)
    if dataVersion:
        response.headers.set("Data-Version", dataVersion)

    if body != None:
        return body
    return json.dumps(responseBody)

def fail(response, body):
    response.status = (400, "Bad Request")
    response.headers.set(b"Content-Type", b"text/plain")
    return body
