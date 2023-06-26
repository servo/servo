"""Automatic beacon store server.

- When a request body is not specified, serves a 200 response whose body
  contains the stored value from the last automatic beacon. If the stored value
  doesn't exist, serves a 200 response with an empty body.
- When a request body is specified, stores the data in the body and serves a 200
  response without body.
"""
# Use an arbitrary key since `request.server.stash.put` expects a valid UUID.
BEACON_KEY = "0c02dba4-f01e-11ed-a05b-0242ac120003"

def main(request, response):
    # Requests with a body imply they were sent as an automatic beacon for
    # reserved.top_navigation. Note that this only stores the most recent beacon
    # that was sent.
    if request.body:
        request.server.stash.put(BEACON_KEY, request.body)
        return (200, [], b"")

    # Requests without a body imply they were sent as the request from
    # nextAutomaticBeacon().
    data = request.server.stash.take(BEACON_KEY)
    if not data and data != "":
        return (200, [], b"<Not set>")
    return (200, [], data)
