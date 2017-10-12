import json

def build_stash_key(session_id, test_num):
    return "%s_%s" % (session_id, test_num)

def main(request, response):
    """Helper handler for Beacon tests.

    It handles two forms of requests:

    STORE:
        A URL with a query string of the form 'cmd=store&sid=<token>&tidx=<test_index>&tid=<test_name>'.

        Stores the receipt of a sendBeacon() request along with its validation result, returning HTTP 200 OK.

        Parameters:
            tidx - the integer index of the test.
            tid - a friendly identifier or name for the test, used when returning results.

    STAT:
        A URL with a query string of the form 'cmd=stat&sid=<token>&tidx_min=<min_test_index>&tidx_max=<max_test_index>'.

        Retrieves the results of test with indices [min_test_index, max_test_index] and returns them as
        a JSON array and HTTP 200 OK status code. Due to the eventual read-once nature of the stash, results for a given test
        are only guaranteed to be returned once, though they may be returned multiple times.

        Parameters:
            tidx_min - the lower-bounding integer test index.
            tidx_max - the upper-bounding integer test index.

        Example response body:
            [{"id": "Test1", error: null}, {"id": "Test2", error: "some validation details"}]

    Common parameters:
        cmd - the command, 'store' or 'stat'.
        sid - session id used to provide isolation to a test run comprising multiple sendBeacon()
              tests.
    """

    session_id = request.GET.first("sid");
    command = request.GET.first("cmd").lower();

    # Workaround to circumvent the limitation that cache keys
    # can only be UUID's.
    def wrap_key(key, path):
        return (str(path), str(key))
    request.server.stash._wrap_key = wrap_key

    # Append CORS headers if needed.
    if "origin" in request.GET:
        response.headers.set("Access-Control-Allow-Origin", request.GET.first("origin"))
    if "credentials" in request.GET:
        response.headers.set("Access-Control-Allow-Credentials", request.GET.first("credentials"))

    # Handle the 'store' and 'stat' commands.
    if command == "store":
        # The test id is just used to make the results more human-readable.
        test_id = request.GET.first("tid")
        # The test index is used to build a predictable stash key, together
        # with the unique session id, in order to retrieve a range of results
        # later knowing the index range.
        test_idx = request.GET.first("tidx")

        test_data_key = build_stash_key(session_id, test_idx)
        test_data = { "id": test_id, "error": None }

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

            # Confirm the payload size sent matches with the number of characters sent.
            if payload_size != len(payload_parts[1]):
                test_data["error"] = "expected %d characters but got %d" % (payload_size, len(payload_parts[1]))
            else:
                # Confirm the payload contains the correct characters.
                for i in range(0, payload_size):
                    if payload_parts[1][i] != "*":
                        test_data["error"] = "expected '*' at index %d but got '%s''" % (i, payload_parts[1][i])
                        break

        # Store the result in the stash so that it can be retrieved
        # later with a 'stat' command.
        request.server.stash.put(test_data_key, test_data)
    elif command == "stat":
        test_idx_min = int(request.GET.first("tidx_min"))
        test_idx_max = int(request.GET.first("tidx_max"))

        # For each result that has come in, append it to the response.
        results = []
        for test_idx in range(test_idx_min, test_idx_max+1): # +1 because end is exclusive
            test_data_key = build_stash_key(session_id, test_idx)
            test_data = request.server.stash.take(test_data_key)
            if test_data:
                results.append(test_data)

        response.headers.set("Content-Type", "text/plain")
        response.content = json.dumps(results)
    else:
        response.status = 400 # BadRequest