import os, json

def main(request, response):
    script_directory = os.path.dirname(os.path.abspath(__file__))
    template_basename = "no-referrer-policy.subresource.template.html"
    template_filename = os.path.join(script_directory, template_basename);

    with open(template_filename) as f:
        template = f.read()

    headers_as_json = json.dumps(request.headers)
    exported_headers = "var SERVER_REQUEST_HEADERS = " + headers_as_json + ";"
    rendered_html = template % {"headers": headers_as_json}

    return response.headers, rendered_html
