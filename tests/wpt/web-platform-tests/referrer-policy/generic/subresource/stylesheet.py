import os, sys, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(request, server_data):
    return subresource.get_template("stylesheet.css.template") % {"id": request.GET["id"]}

def generate_import_rule(request, server_data):
    data = "@import url('%(url)s?id=%(id)s');" % {
        "id": request.GET["id"],
        "url": subresource.create_redirect_url(request, cross_origin = True)
    }
    return data

def main(request, response):
    payload_generator = lambda data: generate_payload(request, data)
    if "import-rule" in request.GET:
        payload_generator = lambda data: generate_import_rule(request, data)

    subresource.respond(
        request,
        response,
        payload_generator = payload_generator,
        content_type = "text/css",
        maybe_additional_headers = { "Referrer-Policy": "unsafe-url" })

