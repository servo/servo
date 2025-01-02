import json

from fledge.tentative.resources import fledge_http_server_util


# Script to generate trusted scoring signals. The responses depends on the
# query strings in the ads Urls - some result in entire response failures,
# others affect only their own value. Each renderUrl potentially has a
# signalsParam, which is a comma-delimited list of instructions that can
# each affect either the value associated with the renderUrl, or the
# response as a whole.
def main(request, response):
    try:
        params = fledge_http_server_util.decode_trusted_scoring_signals_params(request)
    except ValueError as ve:
        return fail(response, str(ve))

    response.status = (200, b"OK")

    # The JSON representation of this is used as the response body.
    responseBody = {"renderUrls": {}}

    # Set when certain special keys are observed, used in place of the JSON
    # representation of `responseBody`, when set.
    body = None

    contentType = "application/json"
    adAuctionAllowed = "true"
    dataVersion = None
    cors = False
    for urlList in params.urlLists:
        for renderUrl in urlList["urls"]:
            value = "default value"
            addValue = True

            try:
                signalsParams = fledge_http_server_util.decode_render_url_signals_params(renderUrl)
            except ValueError as ve:
                return fail(response, str(ve))

            for signalsParam in signalsParams:
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
                    value = params.hostname
                elif signalsParam == "headers":
                    value = fledge_http_server_util.headers_to_ascii(request.headers)
                elif signalsParam == "url":
                    value = request.url
                elif signalsParam == "cors":
                    cors = True
            if addValue:
                if urlList["type"] not in responseBody:
                    responseBody[urlList["type"]] = {}
                responseBody[urlList["type"]][renderUrl] = value

    # If the signalsParam embedded inside a render URL calls for CORS, add
    # appropriate response headers, and fully handle preflights.
    if cors and fledge_http_server_util.handle_cors_headers_and_preflight(
            request, response):
        return

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
