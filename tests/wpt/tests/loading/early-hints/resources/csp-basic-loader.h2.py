import os


def _calculate_csp_value(policy, resource_origin):
    if policy == "absent":
        return None
    elif policy == "allowed":
        return "script-src 'self' 'unsafe-inline' {}".format(resource_origin)
    elif policy == "disallowed":
        return "script-src 'self' 'unsafe-inline'"
    else:
        return None


def handle_headers(frame, request, response):
    resource_origin = request.GET.first(b"resource-origin").decode()

    # Send a 103 response.
    resource_url = request.GET.first(b"resource-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(resource_url)
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    early_hints_csp = _calculate_csp_value(
        request.GET.first(b"early-hints-policy").decode(), resource_origin)
    if early_hints_csp:
        early_hints.append((b"content-security-policy", early_hints_csp))
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Send the final response header.
    response.status = 200
    response.headers["content-type"] = "text/html"
    final_csp = _calculate_csp_value(
        request.GET.first(b"final-policy").decode(), resource_origin)
    if final_csp:
        response.headers["content-security-policy"] = final_csp
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "csp-basic.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
