import os


def handle_headers(frame, request, response):
    # Send two Early Hints responses.

    first_preload = request.GET.first(b"first-preload").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(first_preload)
    early_hints = [
        (b":status", b"103"),
        (b"content-security-policy", "script-src 'self' 'unsafe-inline'"),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    second_preload = request.GET.first(b"second-preload").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(second_preload)
    second_preload_origin = request.GET.first(b"second-preload-origin").decode()
    csp_value = "script-src 'self' 'unsafe-inline' {}".format(second_preload_origin)
    early_hints = [
        (b":status", b"103"),
        (b"content-security-policy", csp_value),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "multiple-early-hints-responses.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
