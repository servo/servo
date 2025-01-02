import os


def handle_headers(frame, request, response):
    # Send an early hints response with an unsupported header.
    # User agents should ignore it.
    early_hints = [
        (b":status", b"103"),
        (b"x-arbitrary-header", b"foobar"),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "arbitrary-header-in-early-hints.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
