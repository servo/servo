import os, sys
from wptserve.utils import isomorphic_decode
sys.path.insert(0, os.path.dirname(os.path.abspath(isomorphic_decode(__file__))))
import subresource

def generate_payload(server_data):
    data = (u'{"headers": %(headers)s}') % server_data
    return data

def main(request, response):
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload,
                        access_control_allow_origin = b"*",
                        content_type = b"application/json")
