# HTTP/2 handler for testing Early Hints with Connection Allowlist.
import os
import time

LINK_ATTRIBUTES = {
    "preload":       b"; rel=preload; as=script",
    "modulepreload": b"; rel=modulepreload; crossorigin",
    "preconnect":    b"; rel=preconnect; crossorigin"
}

def handle_headers(frame, request, response):
    type_param = request.GET.first(b"type", b"").decode("utf-8")
    allow_param = request.GET.first(b"allow", b"").decode("utf-8")
    if (type_param not in ["preload", "modulepreload", "preconnect"] or
            allow_param not in ["true", "false"]):
        return

    # `key` is a UUID for storing a value in the key-value server for verifying
    # whether a request is allowed by the connection allowlist.
    key = request.GET.first(b"key", b"").decode("utf-8")

    # Get WPT config.
    config = request.server.config

    # The base cross-origin domain configured for WPT (resolves to 127.0.0.1 but
    # is treated as a separate origin by the browser).
    alt_host_base = config["domains"]["alt"][""].encode("utf-8")

    # 103 Early Hints must be used over HTTP/2 or HTTP/3.
    #
    # In HTTP/2, the browser can reuse (coalesce) an existing TLS connection for
    # a different hostname if it resolves to the same IP, port, and uses a
    # compatible certificate.
    #
    # If connection coalescing happens, a preconnect test case might reuse a
    # connection opened by a previous test case. This may fail the tests.
    #
    # To prevent coalescing for preconnect:
    # 1. Use the HTTP/1.1 HTTPS port (config["ports"]["https"][0]) for the
    #    preconnect URL. HTTP/1.1 does not support coalescing and requires a
    #    distinct connection per host.
    # 2. Construct a unique hostname for each test run by prepending the unique
    #    'key' to the alt host base: "{key}.[alt-host]".
    https_port = config["ports"]["https"][0]
    unique_alt_host = key.encode("utf-8") + b"." + alt_host_base
    target_origin = b"https://%s:%d" % (unique_alt_host, https_port)

    # Construct the Early Hints response.
    early_hints = [(b":status", b"103")]

    # Construct the Link header in Early Hints response.
    if type_param in ["preload", "modulepreload"]:
        # Preload and modulepreload requests can be verified using the key-value
        # server.
        link_url = (
            target_origin
            + b"/connection-allowlist/tentative/resources"
            + b"/key-value-store.py?key="
            + key.encode("utf-8")
            + b"&value=hello"
        )
    else:
        # Preconnect cannot be verified using the key-value server. See
        # early-hints-preconnect.html which uses Resource Timing API to verify
        # the preconnect.
        link_url = target_origin

    link = b"<" + link_url + b">" + LINK_ATTRIBUTES.get(type_param)

    # Construct the Connection Allowlist header in Early Hints response.
    if allow_param == "true":
        allowlist = b'(response-origin "' + target_origin + b'")'
    else:
        allowlist = b'(response-origin)'

    early_hints.append((b"connection-allowlist", allowlist))
    early_hints.append((b"link", link))

    # Send the Early Hints response.
    response.writer.write_raw_header_frame(headers=early_hints, end_headers=True)

    # Delay before sending the 200 response.
    time.sleep(0.2)

    response.status = 200
    response.headers[b"content-type"] = "text/html"

    # The 200 response does not have a connection allowlist to allow requests
    # to the key-value server for test verification.
    response.write_status_headers()

def main(request, response):
    type_param = request.GET.first(b"type", b"").decode("utf-8")
    if type_param in ["preload", "modulepreload"]:
        filename = "early-hints-preload-modulepreload.html"
    elif type_param == "preconnect":
        filename = "early-hints-preconnect.html"
    else:
        response.status = 400
        response.write_status_headers()
        return

    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, filename)
    with open(file_path, "rb") as f:
        response.writer.write_data(item=f.read(), last=True)
