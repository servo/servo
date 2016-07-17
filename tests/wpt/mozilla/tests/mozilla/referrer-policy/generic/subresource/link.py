import os, json, sys, urllib2, urlparse
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import mozresource; subresource = mozresource

parsed = ''

def generate_payload(server_data):
    # TODO When HTMLLinkElement.sheet is available, don't need this
    #      workaround anymore.
    global parsed

    params = urlparse.parse_qs(parsed.query)
    stash_id = params['id'][0]

    req = urllib2.Request('http://127.0.0.1:8000/_mozilla/mozilla/referrer-policy/generic/subresource/stash.py?id=%s' % stash_id)
    urllib2.urlopen(req, json.dumps(server_data))

    return subresource.get_template("css.template") % server_data

def main(request, response):
    global parsed

    parsed = urlparse.urlparse(request.url)
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload)
