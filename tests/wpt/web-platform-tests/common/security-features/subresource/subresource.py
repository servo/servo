import os, json, urllib, urlparse

def get_template(template_basename):
    script_directory = os.path.dirname(os.path.abspath(__file__))
    template_directory = os.path.abspath(os.path.join(script_directory,
                                                      "template"))
    template_filename = os.path.join(template_directory, template_basename);

    with open(template_filename, "r") as f:
        return f.read()


def redirect(url, response):
    response.add_required_headers = False
    response.writer.write_status(301)
    response.writer.write_header("access-control-allow-origin", "*")
    response.writer.write_header("location", url)
    response.writer.end_headers()
    response.writer.write("")


# TODO(kristijanburnik): subdomain_prefix is a hardcoded value aligned with
# referrer-policy-test-case.js. The prefix should be configured in one place.
def __get_swapped_origin_netloc(netloc, subdomain_prefix = "www1."):
    if netloc.startswith(subdomain_prefix):
        return netloc[len(subdomain_prefix):]
    else:
        return subdomain_prefix + netloc


# Creates a URL (typically a redirect target URL) that is the same as the
# current request URL `request.url`, except for:
# - When `swap_scheme` or `swap_origin` is True, its scheme/origin is changed
#   to the other one. (http <-> https, ws <-> wss, etc.)
# - For `downgrade`, we redirect to a URL that would be successfully loaded
#   if and only if upgrade-insecure-request is applied.
# - `query_parameter_to_remove` parameter is removed from query part.
#   Its default is "redirection" to avoid redirect loops.
def create_url(request,
               swap_scheme=False,
               swap_origin=False,
               downgrade=False,
               query_parameter_to_remove="redirection"):
    parsed = urlparse.urlsplit(request.url)
    destination_netloc = parsed.netloc

    scheme = parsed.scheme
    if swap_scheme:
        scheme = "http" if parsed.scheme == "https" else "https"
        hostname = parsed.netloc.split(':')[0]
        port = request.server.config["ports"][scheme][0]
        destination_netloc = ":".join([hostname, str(port)])

    if downgrade:
        # These rely on some unintuitive cleverness due to WPT's test setup:
        # 'Upgrade-Insecure-Requests' does not upgrade the port number,
        # so we use URLs in the form `http://[domain]:[https-port]`,
        # which will be upgraded to `https://[domain]:[https-port]`.
        # If the upgrade fails, the load will fail, as we don't serve HTTP over
        # the secure port.
        if parsed.scheme == "https":
            scheme = "http"
        elif parsed.scheme == "wss":
            scheme = "ws"
        else:
            raise ValueError("Downgrade redirection: Invalid scheme '%s'" %
                             parsed.scheme)
        hostname = parsed.netloc.split(':')[0]
        port = request.server.config["ports"][parsed.scheme][0]
        destination_netloc = ":".join([hostname, str(port)])

    if swap_origin:
        destination_netloc = __get_swapped_origin_netloc(destination_netloc)

    parsed_query = urlparse.parse_qsl(parsed.query, keep_blank_values=True)
    parsed_query = filter(lambda x: x[0] != query_parameter_to_remove,
                          parsed_query)

    destination_url = urlparse.urlunsplit(urlparse.SplitResult(
        scheme = scheme,
        netloc = destination_netloc,
        path = parsed.path,
        query = urllib.urlencode(parsed_query),
        fragment = None))

    return destination_url


def preprocess_redirection(request, response):
    if "redirection" not in request.GET:
        return False

    redirection = request.GET["redirection"]

    if redirection == "no-redirect":
        return False
    elif redirection == "keep-scheme":
        redirect_url = create_url(request, swap_scheme=False)
    elif redirection == "swap-scheme":
        redirect_url = create_url(request, swap_scheme=True)
    elif redirection == "downgrade":
        redirect_url = create_url(request, downgrade=True)
    elif redirection == "keep-origin":
        redirect_url = create_url(request, swap_origin=False)
    elif redirection == "swap-origin":
        redirect_url = create_url(request, swap_origin=True)
    else:
        raise ValueError("Invalid redirection type '%s'" % redirection)

    redirect(redirect_url, response)
    return True


def preprocess_stash_action(request, response):
    if "action" not in request.GET:
        return False

    action = request.GET["action"]

    key = request.GET["key"]
    stash = request.server.stash
    path = request.GET.get("path", request.url.split('?'))[0]

    if action == "put":
        value = request.GET["value"]
        stash.take(key=key, path=path)
        stash.put(key=key, value=value, path=path)
        response_data = json.dumps({"status": "success", "result": key})
    elif action == "purge":
        value = stash.take(key=key, path=path)
        return False
    elif action == "take":
        value = stash.take(key=key, path=path)
        if value is None:
            status = "allowed"
        else:
            status = "blocked"
        response_data = json.dumps({"status": status, "result": value})
    else:
        return False

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("content-type", "text/javascript")
    response.writer.write_header("cache-control", "no-cache; must-revalidate")
    response.writer.end_headers()
    response.writer.write(response_data)
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

    if preprocess_stash_action(request, response):
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


