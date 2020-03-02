import json


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

    id = request.GET.first("id")
    command = request.GET.first("cmd").lower()

    # Append CORS headers if needed.
    if "origin" in request.GET:
        response.headers.set("Access-Control-Allow-Origin",
                             request.GET.first("origin"))
    if "credentials" in request.GET:
        response.headers.set("Access-Control-Allow-Credentials",
                             request.GET.first("credentials"))

    # Handle the 'store' and 'stat' commands.
    if command == "store":
        error = None

        # Only store the actual POST requests, not any preflight/OPTIONS
        # requests we may get.
        if request.method == "POST":
            payload = ""
            if "Content-Type" in request.headers and \
               "form-data" in request.headers["Content-Type"]:
                if "payload" in request.POST:
                    # The payload was sent as a FormData.
                    payload = request.POST.first("payload")
                else:
                    # A FormData was sent with an empty payload.
                    pass
            else:
                # The payload was sent as either a string, Blob, or BufferSource.
                payload = request.body

            payload_parts = filter(None, payload.split(":"))
            if len(payload_parts) > 0:
                payload_size = int(payload_parts[0])

                # Confirm the payload size sent matches with the number of
                # characters sent.
                if payload_size != len(payload_parts[1]):
                    error = "expected %d characters but got %d" % (
                        payload_size, len(payload_parts[1]))
                else:
                    # Confirm the payload contains the correct characters.
                    for i in range(0, payload_size):
                        if payload_parts[1][i] != "*":
                            error = "expected '*' at index %d but got '%s''" % (
                                i, payload_parts[1][i])
                            break

            # Store the result in the stash so that it can be retrieved
            # later with a 'stat' command.
            request.server.stash.put(id, {"error": error})
        elif request.method == "OPTIONS":
            # If we expect a preflight, then add the cors headers we expect,
            # otherwise log an error as we shouldn't send a preflight for all
            # requests.
            if "preflightExpected" in request.GET:
                response.headers.set("Access-Control-Allow-Headers",
                                     "content-type")
                response.headers.set("Access-Control-Allow-Methods", "POST")
            else:
                error = "Preflight not expected."
                request.server.stash.put(id, {"error": error})
    elif command == "stat":
        test_data = request.server.stash.take(id)
        results = [test_data] if test_data else []

        response.headers.set("Content-Type", "text/plain")
        response.content = json.dumps(results)
    else:
        response.status = 400  # BadRequest
