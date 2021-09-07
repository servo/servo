import json
from wptserve.utils import isomorphic_decode

# A server providing a cacheable response and storing the request's headers
# toward the `uuid` attribute stash.
def main(request, response):
    # The response served is cacheable by the navigator:
    response.headers.set(b"Cache-Control", b"max-age=31536000");

    uuid = request.GET[b'uuid'];
    headers = {};
    for key, value in request.headers.items():
        headers[isomorphic_decode(key)] = isomorphic_decode(request.headers[key])
    headers = json.dumps(headers);

    # The stash is accessed concurrently by many clients. A lock is used to
    # avoid unterleaved read/write from different clients.
    stash = request.server.stash;
    with stash.lock:
        queue = stash.take(uuid, '/coep-credentialless') or [];
        queue.append(headers);
        stash.put(uuid, queue, '/coep-credentialless');
    return b"done";

