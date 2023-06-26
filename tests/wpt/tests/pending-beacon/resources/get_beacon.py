"""An HTTP request handler for WPT that handles /get_beacon.py requests."""

import json

_BEACON_ID_KEY = b"uuid"
_BEACON_DATA_PATH = "beacon_data"


def main(request, response):
    """Retrieves the beacon data keyed by the given uuid from server storage.

    The response content is a JSON string in one of the following formats:
      - "{'data': ['abc', null, '123',...]}"
      - "{'data': []}" indicates that no data has been set for this uuid.
    """
    if _BEACON_ID_KEY not in request.GET:
        response.status = 400
        return "Must provide a UUID to store beacon data"
    uuid = request.GET.first(_BEACON_ID_KEY)

    with request.server.stash.lock:
        body = {'data': []}
        data = request.server.stash.take(key=uuid, path=_BEACON_DATA_PATH)
        if data:
            body['data'] = data
            # The stash is read-once/write-once, so it has to be put back after
            # reading if `data` is not None.
            request.server.stash.put(
                key=uuid, value=data, path=_BEACON_DATA_PATH)
        return [(b'Content-Type', b'text/plain')], json.dumps(body)
