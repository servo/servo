import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(server_data):
    return subresource.get_template("document.html.template") % server_data

def main(request, response):
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload)
