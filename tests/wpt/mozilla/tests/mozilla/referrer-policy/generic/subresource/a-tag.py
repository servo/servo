import os, json, sys
this_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, this_dir)

import mozresource; subresource = mozresource

def generate_payload(server_data):
    with open(os.path.join(this_dir, "a-tag.html")) as f:
        return f.read()

def main(request, response):
    path = 'a-tag-stash'
    server_data = json.dumps(request.GET)
    stashed_data = request.server.stash.take(request.GET["id"], path)
    if stashed_data:
        server_data = stashed_data
    request.server.stash.put(request.GET["id"], server_data, path)
    http_header_referrer_policy = request.GET["httpReferrer"] if "httpReferrer" in request.GET else None
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload,
                        http_header_referrer_policy = http_header_referrer_policy)
