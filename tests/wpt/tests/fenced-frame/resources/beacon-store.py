"""
Event beacon store server.

- When a request body is specified, stores the data in the body for the 'type'
  specified in the query parameters and serves a 200 response without body.
- When a request body is not specified and the request is not served with an
  'expected_body' parameter, stores an empty body for the 'type' specified in
  the query parameters and serves a 200 response without body.
- When a request body is not specified and the request is served with an
  'expected_body' parameter, serves a 200 response whose body contains the
  stored value from the automatic beacon. Since the data is stored using a hash
  of the data as the key, it uses the `expected_body` query parameter to know
  what key to look up. If the stored value doesn't exist, serves a 200 response
  with an empty body.
"""
import uuid
import hashlib

NO_DATA_STRING = b"<No data>"
NOT_SET_STRING = b"<Not set>"

# The server stash requires a uuid to store data. Use a hash of the automatic
# beacon data as the uuid to store and retrieve the data.
def string_to_uuid(input):
    hash_value = hashlib.md5(str(input).encode("UTF-8")).hexdigest()
    return str(uuid.UUID(hex=hash_value))

def main(request, response):
    stash = request.server.stash;
    event_type = request.GET.first(b"type", NO_DATA_STRING)

    # The stash is accessed concurrently by many clients. A lock is used to
    # avoid interleaved read/write from different clients.
    with stash.lock:
        # GET requests with an 'expected_body' parameter imply they were sent as
        # the request from nextBeacon().
        if request.method == "GET" and b"expected_body" in request.GET:
          expected_body = request.GET.first(b"expected_body", NO_DATA_STRING)
          data = stash.take(string_to_uuid(event_type + expected_body)) or NOT_SET_STRING
          return (200, [], data)

        # Requests with a body imply they were sent as a reporting beacon
        # (either through reportEvent() or through an automatic beacon).
        if request.method == "POST" and event_type:
            request_body = request.body or NO_DATA_STRING
            request_origin = request.headers.get("Origin") or NO_DATA_STRING
            request_referrer = request.headers.get("Referer") or NO_DATA_STRING
            stash.put(string_to_uuid(event_type + request_body),
                (request_origin + b"," + request_referrer))
            return (200, [], b"")
        # GET requests without an 'expected_body' parameter imply they were sent
        # as a destination URL reporting beacon.
        if request.method == "GET" and event_type:
            request_origin = request.headers.get("Origin") or NO_DATA_STRING
            request_referrer = request.headers.get("Referer") or NO_DATA_STRING
            stash.put(string_to_uuid(event_type + NO_DATA_STRING),
                (request_origin + b"," + request_referrer))
            return (200, [], b"")

        return (400, [], u"")
