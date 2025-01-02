import os


def handle_headers(frame, request, response):
    # Send a 103 response.
    resource_origin = request.GET.first(b"resource-origin").decode()
    link_header_value = "<{}>; rel=preconnect".format(resource_origin)
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Send the final response header.
    response.status = 200
    response.headers["content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "preconnect-in-early-hints.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
