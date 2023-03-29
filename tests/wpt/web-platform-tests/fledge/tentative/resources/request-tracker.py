import mimetypes
import os
import json
import wptserve.stash

from wptserve.utils import isomorphic_decode, isomorphic_encode

# Test server that tracks requests it has previously seen, keyed by a token.
#
# All requests have a "dispatch" field indicating what to do, and a "uuid"
# field which should be unique for each test, to avoid tests that fail to
# clean up after themselves, or that are running concurrently, from interfering
# with other tests.
#
# Each uuid has a stash entry with a dictionary with two entries:
#     "trackedRequests" is a list of all observed requested URLs with a
#         dispatch of "track_get" or "track_post". POSTS are in the format
#         "<url>, body: <body>".
#     "errors" is a list of an errors that occurred.
#
# A dispatch of "request_list" will return the "trackedRequests" dictionary
# associated with the in uuid, as a JSON string.
#
# A dispatch of "clean_up" will delete all information associated with the uuid.
def main(request, response):
    # Don't cache responses, since tests expect duplicate requests to always
    # reach the server.
    response.headers.set(b"Cache-Control", b"no-store")

    dispatch = request.GET.first(b"dispatch", None)
    uuid = request.GET.first(b"uuid", None)

    if not uuid or not dispatch:
        return simple_response(request, response, 404, b"Not found",
                               b"Invalid query parameters")

    stash = request.server.stash
    with stash.lock:
        # Take ownership of stashed entry, if any. This removes the entry of the
        # stash.
        server_state = stash.take(uuid) or {"trackedRequests": [], "errors": []}

        # Clear the entire stash. No work to do, since stash entry was already
        # removed.
        if dispatch == b"clean_up":
            return simple_response(request, response, 200, b"OK",
                                   b"cleanup complete")

        # Return the list of entries in the stash. Need to add data back to the
        # stash first.
        if dispatch == b"request_list":
            stash.put(uuid, server_state)
            return simple_response(request, response, 200, b"OK",
                                   json.dumps(server_state))

        # Tracks a request that's expected to be a GET.
        if dispatch == b"track_get":
            if request.method != "GET":
                server_state["errors"].append(
                    request.url + " has wrong method: " + request.method)
            else:
                server_state["trackedRequests"].append(request.url)

            stash.put(uuid, server_state)
            return simple_response(request, response, 200, b"OK", b"")

        # Tracks a request that's expected to be a POST.
        # In addition to the method, check the Content-Type, which is currently
        # always text/plain, and compare the body against the expected body.
        if dispatch == b"track_post":
            contentType = request.headers.get(b"Content-Type", b"missing")
            if request.method != "POST":
                server_state["errors"].append(
                    request.url + " has wrong method: " + request.method)
            elif not contentType.startswith(b"text/plain"):
                server_state["errors"].append(
                    request.url + " has wrong Content-Type: " +
                    contentType.decode("utf-8"))
            else:
                server_state["trackedRequests"].append(
                    request.url + ", body: " + request.body.decode("utf-8"))
            stash.put(uuid, server_state)
            return simple_response(request, response, 200, b"OK", b"")

        # Report unrecognized dispatch line.
        server_state["errors"].append(
            request.url + " request with unknown dispatch value received: " +
            dispatch.decode("utf-8"))
        stash.put(uuid, server_state)
        return simple_response(request, response, 404, b"Not Found",
                               b"Unrecognized dispatch parameter: " + dispatch)

def simple_response(request, response, status_code, status_message, body,
                    content_type=b"text/plain"):
    response.status = (status_code, status_message)
    response.headers.set(b"Content-Type", content_type)
    return body
