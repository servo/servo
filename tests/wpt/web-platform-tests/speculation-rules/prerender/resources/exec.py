from wptserve.utils import isomorphic_decode
import os

def main(request, response):
    purpose = request.headers.get(b"purpose")
    if (purpose == b'prefetch' and b"code" in request.GET):
        code = int(request.GET.first(b"code"))
    else:
        code = 200

    if b"uuid" in request.GET:
        path = '/speculation-rules/prerender/resources/exec.py'
        uuid = request.GET.first(b"uuid")
        with request.server.stash.lock:
            count = request.server.stash.take(uuid, path) or 0
            if b"get-fetch-count" in request.GET:
                response.status = 200
                response.content = '%d' % count
                request.server.stash.put(uuid, count, path)
                return
            count += 1
            request.server.stash.put(uuid, count, path)

    with open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), "exec.html"), u"r") as fn:
        response.content = fn.read()
    response.status = code
