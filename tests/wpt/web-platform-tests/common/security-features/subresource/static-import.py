import os, sys, urllib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(request):
    import_url = urllib.unquote(request.GET['import_url'])
    return subresource.get_template("static-import.js.template") % {
        "import_url": import_url
    }

def main(request, response):
    payload_generator = lambda _: generate_payload(request)
    subresource.respond(request,
                        response,
                        payload_generator = payload_generator,
                        content_type = "application/javascript")
