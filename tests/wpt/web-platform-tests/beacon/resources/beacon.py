import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    """Helper handler for Beacon tests.

    It handles two forms of requests:

    STORE:
        A URL with a query string of the form 'cmd=store&id=<token>'.

        Stores the receipt of a sendBeacon() request along with its validation
        result, returning HTTP 200 OK.

        if "preflightExpected"  exists in the query, this handler responds to
        CORS preflights.

    STAT:
        A URL with a query string of the form 'cmd=stat&id=<token>'.

        Retrieves the results of test for the given id and returns them as a
        JSON array and HTTP 200 OK status code. Due to the eventual read-once
        nature of the stash, results for a given test are only guaranteed to be
        returned once, though they may be returned multiple times.

        Example response bodies:
            - [{error: null}]
            - [{error: "some validation details"}]
            - []

    Common parameters:
        cmd - the command, 'store' or 'stat'.
        id - the unique identifier of the test.
    """

    id = request.GET.first(b"id")
    command = request.GET.first(b"cmd").lower()

    # Append CORS headers if needed.
    if b"origin" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin",
                             request.GET.first(b"origin"))
    if b"credentials" in request.GET:
        response.headers.set(b"Access-Control-Allow-Credentials",
                             request.GET.first(b"credentials"))

    # Handle the 'store' and 'stat' commands.
    if command == b"store":
        error = None

        # Only store the actual POST requests, not any preflight/OPTIONS
        # requests we may get.
        if request.method == u"POST":
            payload = b""
            if b"Content-Type" in request.headers and \
               b"form-data" in request.headers[b"Content-Type"]:
                if b"payload" in request.POST:
                    # The payload was sent as a FormData.
                    payload = request.POST.first(b"payload")
                else:
                    # A FormData was sent with an empty payload.
                    pass
            else:
                # The payload was sent as either a string, Blob, or BufferSource.
                payload = request.body

            payload_parts = list(filter(None, payload.split(b":")))
            if len(payload_parts) > 0:
                payload_size = int(payload_parts[0])

                # Confirm the payload size sent matches with the number of
                # characters sent.
                if payload_size != len(payload_parts[1]):
                    error = u"expected %d characters but got %d" % (
                        payload_size, len(payload_parts[1]))
                else:
                    # Confirm the payload contains the correct characters.
                    for i in range(0, payload_size):
                        if payload_parts[1][i:i+1] != b"*":
                            error = u"expected '*' at index %d but got '%s''" % (
                                i, isomorphic_decode(payload_parts[1][i:i+1]))
                            break

            # Store the result in the stash so that it can be retrieved
            # later with a 'stat' command.
            request.server.stash.put(id, {u"error": error})
        elif request.method == u"OPTIONS":
            # If we expect a preflight, then add the cors headers we expect,
            # otherwise log an error as we shouldn't send a preflight for all
            # requests.
            if b"preflightExpected" in request.GET:
                response.headers.set(b"Access-Control-Allow-Headers",
                                     b"content-type")
                response.headers.set(b"Access-Control-Allow-Methods", b"POST")
            else:
                error = u"Preflight not expected."
                request.server.stash.put(id, {u"error": error})
    elif command == b"stat":
        test_data = request.server.stash.take(id)
        results = [test_data] if test_data else []

        response.headers.set(b"Content-Type", b"text/plain")
        response.content = json.dumps(results)
    else:
        response.status = 400  # BadRequest
