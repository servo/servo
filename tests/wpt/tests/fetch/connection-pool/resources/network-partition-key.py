import mimetypes
import os

from wptserve.utils import isomorphic_decode, isomorphic_encode

# Test server that tracks the last partition_id was used with each connection for each uuid, and
# lets consumers query if multiple different partition_ids have been been used for any socket.
#
# Server assumes that ports aren't reused, so a client address and a server port uniquely identify
# a connection. If that constraint is ever violated, the test will be flaky. No sockets being
# closed for the duration of the test is sufficient to ensure that, though even if sockets are
# closed, the OS should generally prefer to use new ports for new connections, if any are
# available.
def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    dispatch = request.GET.first(b"dispatch", None)
    uuid = request.GET.first(b"uuid", None)
    partition_id = request.GET.first(b"partition_id", None)

    if not uuid or not dispatch or not partition_id:
        return simple_response(request, response, 404, b"Not found", b"Invalid query parameters")

    # Unless nocheck_partition is true, check partition_id against server_state, and update server_state.
    stash = request.server.stash
    test_failed = False
    request_count = 0;
    connection_count = 0;
    if request.GET.first(b"nocheck_partition", None) != b"True":
        # Need to grab the lock to access the Stash, since requests are made in parallel.
        with stash.lock:
            # Don't use server hostname here, since H2 allows multiple hosts to reuse a connection.
            # Server IP is not currently available, unfortunately.
            address_key = isomorphic_encode(str(request.client_address) + u"|" + str(request.url_parts.port))
            server_state = stash.take(uuid) or {b"test_failed": False,
              b"request_count": 0, b"connection_count": 0}
            request_count = server_state[b"request_count"]
            request_count += 1
            server_state[b"request_count"] = request_count
            if address_key in server_state:
                if server_state[address_key] != partition_id:
                    server_state[b"test_failed"] = True
            else:
                connection_count = server_state[b"connection_count"]
                connection_count += 1
                server_state[b"connection_count"] = connection_count
            server_state[address_key] = partition_id
            test_failed = server_state[b"test_failed"]
            stash.put(uuid, server_state)

    origin = request.headers.get(b"Origin")
    if origin:
        response.headers.set(b"Access-Control-Allow-Origin", origin)
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    if request.method == u"OPTIONS":
        return handle_preflight(request, response)

    if dispatch == b"fetch_file":
        return handle_fetch_file(request, response, partition_id, uuid)

    if dispatch == b"check_partition":
        status = request.GET.first(b"status", 200)
        if test_failed:
            return simple_response(request, response, status, b"OK", b"Multiple partition IDs used on a socket")
        body = b"ok"
        if request.GET.first(b"addcounter", False):
            body += (". Request was sent " + str(request_count) + " times. " +
             str(connection_count) + " connections were created.").encode('utf-8')
        return simple_response(request, response, status, b"OK", body)

    if dispatch == b"clean_up":
        stash.take(uuid)
        if test_failed:
            return simple_response(request, response, 200, b"OK", b"Test failed, but cleanup completed.")
        return simple_response(request, response, 200, b"OK", b"cleanup complete")

    return simple_response(request, response, 404, b"Not Found", b"Unrecognized dispatch parameter: " + dispatch)

def handle_preflight(request, response):
    response.status = (200, b"OK")
    response.headers.set(b"Access-Control-Allow-Methods", b"GET")
    response.headers.set(b"Access-Control-Allow-Headers", b"header-to-force-cors")
    response.headers.set(b"Access-Control-Max-Age", b"86400")
    return b"Preflight request"

def simple_response(request, response, status_code, status_message, body, content_type=b"text/plain"):
    response.status = (status_code, status_message)
    response.headers.set(b"Content-Type", content_type)
    return body

def handle_fetch_file(request, response, partition_id, uuid):
    subresource_origin = request.GET.first(b"subresource_origin", None)
    rel_path = request.GET.first(b"path", None)

    # This needs to be passed on to subresources so they all have access to it.
    include_credentials = request.GET.first(b"include_credentials", None)
    if not subresource_origin or not rel_path or not include_credentials:
        return simple_response(request, response, 404, b"Not found", b"Invalid query parameters")

    cur_path = os.path.realpath(isomorphic_decode(__file__))
    base_path = os.path.abspath(os.path.join(os.path.dirname(cur_path), os.pardir, os.pardir, os.pardir))
    path = os.path.abspath(os.path.join(base_path, isomorphic_decode(rel_path)))

    # Basic security check.
    if not path.startswith(base_path):
        return simple_response(request, response, 404, b"Not found", b"Invalid path")

    sandbox = request.GET.first(b"sandbox", None)
    if sandbox == b"true":
        response.headers.set(b"Content-Security-Policy", b"sandbox allow-scripts")

    file = open(path, mode="rb")
    body = file.read()
    file.close()

    subresource_path = b"/" + isomorphic_encode(os.path.relpath(isomorphic_decode(__file__), base_path)).replace(b'\\', b'/')
    subresource_params = b"?partition_id=" + partition_id + b"&uuid=" + uuid + b"&subresource_origin=" + subresource_origin + b"&include_credentials=" + include_credentials
    body = body.replace(b"SUBRESOURCE_PREFIX:", subresource_origin + subresource_path + subresource_params)

    other_origin = request.GET.first(b"other_origin", None)
    if other_origin:
        body = body.replace(b"OTHER_PREFIX:", other_origin + subresource_path + subresource_params)

    mimetypes.init()
    mimetype_pair = mimetypes.guess_type(path)
    mimetype = mimetype_pair[0]

    if mimetype == None or mimetype_pair[1] != None:
        return simple_response(request, response, 500, b"Server Error", b"Unknown MIME type")
    return simple_response(request, response, 200, b"OK", body, mimetype)
