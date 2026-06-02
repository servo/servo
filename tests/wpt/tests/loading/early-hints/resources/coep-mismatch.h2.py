import os


def handle_headers(frame, request, response):
    # Send a 103 response.
    resource_url = request.GET.first(b"resource-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(resource_url)
    coep_value = request.GET.first(b"early-hints-policy").decode()
    early_hints = [
        (b":status", b"103"),
        (b"cross-origin-embedder-policy", coep_value),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Send the final response header.
    coep_value = request.GET.first(b"final-policy").decode()
    response.status = 200
    response.headers["content-type"] = "text/html"
    response.headers["cross-origin-embedder-policy"] = coep_value
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "coep-mismatch.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
