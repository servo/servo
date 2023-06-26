import datetime
import json
import time


def _url_dir(request):
    return u'/'.join(request.url_parts.path.split(u'/')[:-1]) + u'/'


def store_request_timing_and_headers(request):
    """Store the current timestamp and request's headers in the stash object of
    the server. The request must a GET request and must have the "id" parameter.
    """
    id = request.GET.first(b"id")
    timestamp = datetime.datetime.now().timestamp()

    value = {
        "timestamp": timestamp,
        "headers": request.raw_headers,
    }

    url_dir = _url_dir(request)
    request.server.stash.put(id, value, url_dir)


def get_request_timing_and_headers(request, id=None):
    """Get previously stored timestamp and request headers associated with the
    given "id". When "id" is not given the id is retrieved from "request".
    """
    if id is None:
        id = request.GET.first(b"id")
    url_dir = _url_dir(request)
    item = request.server.stash.take(id, url_dir)
    if not item:
        return None
    return json.dumps(item)


def wait_for_preload_to_finish(request, id):
    """Wait until a preload associated with "id" is sent."""
    while True:
        if get_request_timing_and_headers(request, id):
            break
        time.sleep(0.1)
    time.sleep(0.1)
