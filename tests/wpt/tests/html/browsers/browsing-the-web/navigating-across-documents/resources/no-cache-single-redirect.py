# This handler receives requests identified by UUIDs that have mandatory
# `location` query parameters. Every other request for the same URL will result
# in a redirect to the URL described by `location`. When we don't redirect, we
# simply return the HTML document "No redirect".
def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")

    uuid = request.GET.first(b"uuid")
    value = request.server.stash.take(uuid)

    if value is None:
        response.status = 302
        location = request.GET.first(b"location")
        response.headers.set(b"Location", location)
        # Ensure that the next time this uuid is request, we don't redirect.
        request.server.stash.put(uuid, "sentinel value")
    else:
        # If we're in this branch, then `value` is not none, but the stash now
        # has `None` associated with `uuid`, which means on the next request for
        # this `uuid` we'll end up in the above branch instead.
        return ([(b"Content-Type", b"text/html")], b"No redirect")
