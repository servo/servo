import os, json, sys, urllib2, urlparse
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import mozresource; subresource = mozresource

def generate_payload(server_data):
    return subresource.get_template("css.template") % server_data

def main(request, response):
    # TODO: When HTMLLinkElement.sheet is available, don't need this
    # workaround anymore. We could store data into css.template and
    # read it from HTMLLinkElement.sheet.
    path = 'link-element-stash'
    server_data = {"headers": json.dumps(request.headers, indent = 4)}

    # We do this because in those tests which cause redirection, this
    # subresource will be called at least twice. And putting data onto stash
    # more than once is not a valid operation (it'll throw exception). When
    # there's already something in stash, we simply take it out and put the
    # new one back since we only interesting in the last request.
    stashed_data = request.server.stash.take(request.GET["id"], path)

    request.server.stash.put(request.GET["id"], json.dumps(server_data), path)

    subresource.respond(request,
                        response,
                        payload_generator = generate_payload)
