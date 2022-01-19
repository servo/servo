import os, sys
from urllib.parse import unquote

from wptserve.utils import isomorphic_decode
import importlib
subresource = importlib.import_module("common.security-features.subresource.subresource")

def generate_payload(request):
    import_url = unquote(isomorphic_decode(request.GET[b'import_url']))
    return subresource.get_template(u"static-import.js.template") % {
        u"import_url": import_url
    }

def main(request, response):
    payload_generator = lambda _: generate_payload(request)
    subresource.respond(request,
                        response,
                        payload_generator = payload_generator,
                        content_type = b"application/javascript")
