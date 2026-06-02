import os


def handle_headers(frame, request, response):
    # Send a 103 response.
    resource_url = request.GET.first(b"resource-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(resource_url)
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]

    early_hints_policy = request.GET.first(b"early-hints-policy").decode()
    # In this test handler "allowed" or "absent" are only valid policies because
    # csp-document-disallow.html always sets CSP to disallow the preload.
    # "disallowed" makes no observable changes in the test. Note that
    # csp-basic.html covers disallowing preloads in Early Hints.
    assert early_hints_policy == "allowed" or early_hints_policy == "absent"

    if early_hints_policy == "allowed":
        resource_origin = request.GET.first(b"resource-origin").decode()
        csp_value = "script-src 'self' 'unsafe-inline' {}".format(resource_origin)
        early_hints.append((b"content-security-policy", csp_value))

    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Send the final response header.
    response.status = 200
    response.headers["content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "csp-document-disallow.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
