"""An HTTP request handler for WPT that handles /set_beacon.py requests."""

_BEACON_ID_KEY = b"uuid"
_BEACON_DATA_PATH = "beacon_data"
_BEACON_FORM_PAYLOAD_KEY = b"payload"
_BEACON_BODY_PAYLOAD_KEY = "payload="
_BEACON_EXPECT_ORIGIN_KEY = b"expectOrigin"
_BEACON_EXPECT_PREFLIGHT_KEY = b"expectPreflight"
_BEACON_EXPECT_CREDS_KEY = b"expectCredentials"


def main(request, response):
    """Stores the given beacon's data keyed by uuid in the server.

    For GET request, this handler assumes no data.
    For POST request, this handler extracts data from request body:
      - Content-Type=multipart/form-data: data keyed by 'payload'.
      - the entire request body.

    Multiple data can be added for the same uuid.

    The data is stored as UTF-8 format.
    """
    if _BEACON_ID_KEY not in request.GET:
        response.status = 400
        return "Must provide a UUID to store beacon data"
    uuid = request.GET.first(_BEACON_ID_KEY)

    expected_origin = request.GET.get(_BEACON_EXPECT_ORIGIN_KEY)
    if b"origin" in request.headers:
        origin = request.headers.get(b"origin")
        if expected_origin:
            assert origin == expected_origin, f"expected {expected_origin}, got {origin}"
        response.headers.set(b"Access-Control-Allow-Origin", origin)
    else:
        assert expected_origin is None, f"expected None, got {expected_origin}"

    # Handles preflight request first.
    if request.method == u"OPTIONS":
        assert request.GET.get(
            _BEACON_EXPECT_PREFLIGHT_KEY) == b"true", "Preflight not expected."

        # preflight must not have cookies.
        assert b"Cookie" not in request.headers

        requested_headers = request.headers.get(
            b"Access-Control-Request-Headers")
        assert b"content-type" in requested_headers, f"expected content-type, got {requested_headers}"
        response.headers.set(b"Access-Control-Allow-Headers", b"content-type")

        requested_method = request.headers.get(b"Access-Control-Request-Method")
        assert requested_method == b"POST", f"expected POST, got {requested_method}"
        response.headers.set(b"Access-Control-Allow-Methods", b"POST")

        return response

    expect_creds = request.GET.get(_BEACON_EXPECT_CREDS_KEY) == b"true"
    if expect_creds:
        assert b"Cookie" in request.headers
    else:
        assert b"Cookie" not in request.headers

    data = None
    if request.method == u"POST":
        if b"multipart/form-data" in request.headers.get(b"Content-Type", b""):
            if _BEACON_FORM_PAYLOAD_KEY in request.POST:
                data = request.POST.first(_BEACON_FORM_PAYLOAD_KEY).decode(
                    'utf-8')
        elif request.body:
            data = request.body.decode('utf-8')
            if data.startswith(_BEACON_BODY_PAYLOAD_KEY):
                data = data.split(_BEACON_BODY_PAYLOAD_KEY)[1]

    with request.server.stash.lock:
        saved_data = request.server.stash.take(key=uuid, path=_BEACON_DATA_PATH)
        if not saved_data:
            saved_data = [data]
        else:
            saved_data.append(data)
        request.server.stash.put(
            key=uuid, value=saved_data, path=_BEACON_DATA_PATH)

    response.status = 200
