import os, sys
from wptserve.utils import isomorphic_decode
sys.path.insert(0, os.path.dirname(os.path.abspath(isomorphic_decode(__file__))))
import subresource

def generate_payload(request, server_data):
    data = (u'{"headers": %(headers)s}') % server_data
    if b"id" in request.GET:
        with request.server.stash.lock:
            request.server.stash.take(request.GET[b"id"])
            request.server.stash.put(request.GET[b"id"], data)
    return u"<svg xmlns='http://www.w3.org/2000/svg'></svg>"

def generate_payload_embedded(request, server_data):
    return subresource.get_template(u"svg.embedded.template") % {
        b"id": request.GET[b"id"],
        b"property": request.GET[b"property"]}

def generate_report_headers_payload(request, server_data):
    stashed_data = request.server.stash.take(request.GET[b"id"])
    return stashed_data

def main(request, response):
    handler = lambda data: generate_payload(request, data)
    content_type = b'image/svg+xml'

    if b"embedded-svg" in request.GET:
        handler = lambda data: generate_payload_embedded(request, data)

    if b"report-headers" in request.GET:
        handler = lambda data: generate_report_headers_payload(request, data)
        content_type = b'application/json'

    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        content_type = content_type)
