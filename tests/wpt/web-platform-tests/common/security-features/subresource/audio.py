import os, sys
from wptserve.utils import isomorphic_decode
sys.path.insert(0, os.path.dirname(os.path.abspath(isomorphic_decode(__file__))))
import subresource

def generate_payload(request, server_data):
    file = os.path.join(request.doc_root, u"webaudio", u"resources",
                        u"sin_440Hz_-6dBFS_1s.wav")
    return open(file, "rb").read()


def main(request, response):
    handler = lambda data: generate_payload(request, data)
    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        access_control_allow_origin = b"*",
                        content_type = b"audio/wav")
