import os


def handle_headers(frame, request, response):
    header_value = request.GET.first(b"header-value")
    early_hints = [
        (b":status", b"103"),
        (b"invalid-header", header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    test_content = "<div>This page should not be loaded.</div>"
    response.writer.write_data(item=test_content, last=True)
