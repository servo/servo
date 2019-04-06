import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(request, server_data):
    data = ('{"headers": %(headers)s}') % server_data
    if "id" in request.GET:
        with request.server.stash.lock:
            request.server.stash.take(request.GET["id"])
            request.server.stash.put(request.GET["id"], data)
    return "<svg xmlns='http://www.w3.org/2000/svg'></svg>";

def generate_payload_embedded(request, server_data):
    return subresource.get_template("svg.embedded.template") % {
        "id": request.GET["id"],
        "property": request.GET["property"]};

def generate_report_headers_payload(request, server_data):
    stashed_data = request.server.stash.take(request.GET["id"])
    return stashed_data

def main(request, response):
    handler = lambda data: generate_payload(request, data)
    content_type = 'image/svg+xml'

    if "embedded-svg" in request.GET:
        handler = lambda data: generate_payload_embedded(request, data)

    if "report-headers" in request.GET:
        handler = lambda data: generate_report_headers_payload(request, data)
        content_type = 'application/json'

    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        content_type = content_type)
