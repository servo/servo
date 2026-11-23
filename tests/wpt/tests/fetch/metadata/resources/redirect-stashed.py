import hashlib
import json

from wptserve.utils import isomorphic_encode

# wptserve stash keys must be valid UUIDs, so derive one from a readable name
# rather than using the name directly.
#
# The key is fixed because intermediate hops must not carry query parameters,
# which leaves no way for a request to identify which redirect chain it belongs
# to. As a result, every user of this handler shares a single stash entry: tests
# that use it must not run their redirect chains concurrently, or they will
# overwrite or consume each other's stashed locations.
STASH_KEY = hashlib.md5(b"fetch-metadata-stashed-redirect").hexdigest()


def main(request, response):
    """Redirect through a stashed list of URLs without query parameters on intermediate hops."""
    headers = [
        (b"Content-Type", b"text/plain"),
        (b"Cache-Control", b"no-cache, no-store, must-revalidate"),
        (b"Pragma", b"no-cache"),
    ]

    if b"location" in request.GET:
        raw_locations = request.GET.first(b"location")
    elif b"location" in request.POST:
        raw_locations = request.POST.first(b"location")
    else:
        raw_locations = None

    with request.server.stash.lock:
        if raw_locations is not None:
            locations = json.loads(raw_locations)
            request.server.stash.take(STASH_KEY)
        else:
            locations = request.server.stash.take(STASH_KEY)

        if not locations:
            return 400, headers, b"Missing stashed redirect location"

        next_url = locations.pop(0)
        if locations:
            request.server.stash.put(STASH_KEY, locations)

    headers.append((b"Location", isomorphic_encode(next_url)))
    return 302, headers, b""
