import os, sys, json, urlparse, urllib

def get_template(template_basename):
    script_directory = os.path.dirname(os.path.abspath(__file__))
    template_directory = os.path.abspath(os.path.join(script_directory,
                                                      "..",
                                                      "template"))
    template_filename = os.path.join(template_directory, template_basename);

    with open(template_filename, "r") as f:
        return f.read()

# TODO(kristijanburnik): subdomain_prefix is a hardcoded value aligned with
# referrer-policy-test-case.js. The prefix should be configured in one place.
def get_swapped_origin_netloc(netloc, subdomain_prefix = "www1."):
    if netloc.startswith(subdomain_prefix):
        return netloc[len(subdomain_prefix):]
    else:
        return subdomain_prefix + netloc

def create_redirect_url(request, cross_origin = False):
    parsed = urlparse.urlsplit(request.url)
    destination_netloc = parsed.netloc
    if cross_origin:
        destination_netloc = get_swapped_origin_netloc(parsed.netloc)

    destination_url = urlparse.urlunsplit(urlparse.SplitResult(
        scheme = parsed.scheme,
        netloc = destination_netloc,
        path = parsed.path,
        query = None,
        fragment = None))

    return destination_url


def redirect(url, response):
    response.add_required_headers = False
    response.writer.write_status(301)
    response.writer.write_header("access-control-allow-origin", "*")
    response.writer.write_header("location", url)
    response.writer.end_headers()
    response.writer.write("")


def preprocess_redirection(request, response):
    if "redirection" not in request.GET:
        return False

    redirection = request.GET["redirection"]

    if redirection == "no-redirect":
        return False
    elif redirection == "keep-origin-redirect":
        redirect_url = create_redirect_url(request, cross_origin = False)
    elif redirection == "swap-origin-redirect":
        redirect_url = create_redirect_url(request, cross_origin = True)
    else:
        raise ValueError("Invalid redirection type '%s'" % redirection)

    redirect(redirect_url, response)
    return True


def __noop(request, response):
    return ""


def respond(request,
            response,
            status_code = 200,
            content_type = "text/html",
            payload_generator = __noop,
            cache_control = "no-cache; must-revalidate",
            access_control_allow_origin = "*",
            maybe_additional_headers = None):
    if preprocess_redirection(request, response):
        return

    response.add_required_headers = False
    response.writer.write_status(status_code)

    if access_control_allow_origin != None:
        response.writer.write_header("access-control-allow-origin",
                                     access_control_allow_origin)
    response.writer.write_header("content-type", content_type)
    response.writer.write_header("cache-control", cache_control)

    additional_headers = maybe_additional_headers or {}
    for header, value in additional_headers.items():
        response.writer.write_header(header, value)

    response.writer.end_headers()

    server_data = {"headers": json.dumps(request.headers, indent = 4)}

    payload = payload_generator(server_data)
    response.writer.write(payload)


