import os, sys, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(server_data):
    data = ('{"headers": %(headers)s}') % server_data
    return data

def main(request, response):
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload,
                        access_control_allow_origin = "*",
                        content_type = "application/json")

