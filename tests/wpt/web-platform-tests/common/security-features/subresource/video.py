import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(request, server_data):
    file = os.path.join(request.doc_root, "media", "movie_5.ogv")
    return open(file, "rb").read()


def main(request, response):
    handler = lambda data: generate_payload(request, data)
    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        access_control_allow_origin = "*",
                        content_type = "video/ogg")

