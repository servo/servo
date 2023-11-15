"""
Automatic beacon store server.

- When a request body is specified, stores the data in the body and serves a 200
  response without body.
- When a request body is not specified, serves a 200 response whose body
  contains the stored value from the automatic beacon. Since the data is stored
  using a hash of the data as the key, it expects an `expected_body` query
  parameter to know what key to look up. If the stored value doesn't exist,
  serves a 200 response with an empty body.
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

    # The stash is accessed concurrently by many clients. A lock is used to
    # avoid interleaved read/write from different clients.
    with stash.lock:
        # Requests with a body imply they were sent as an automatic beacon. Note
        # that this only stores the most recent beacon that was sent.
        if request.method == "POST":
            request_body = request.body or NO_DATA_STRING
            stash.put(string_to_uuid(request_body), request_body)
            return (200, [], b"")

        # Requests without a body imply they were sent as the request from
        # nextAutomaticBeacon().
        expected_body = request.GET.first(b"expected_body", NO_DATA_STRING)
        data = stash.take(string_to_uuid(expected_body)) or NOT_SET_STRING
        return(200, [], data)
