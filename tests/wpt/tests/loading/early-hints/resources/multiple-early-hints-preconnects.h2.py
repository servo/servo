import os


def handle_headers(frame, request, response):
    # Send two Early Hints responses, each with a preconnect to a different
    # origin. Only the first response should be honored by the browser.
    first = request.GET.first(b"first-preconnect").decode()
    response.writer.write_raw_header_frame(
        headers=[
            (b":status", b"103"),
            (b"link", "<{}>; rel=preconnect".format(first).encode()),
        ],
        end_headers=True)

    second = request.GET.first(b"second-preconnect").decode()
    response.writer.write_raw_header_frame(
        headers=[
            (b":status", b"103"),
            (b"link", "<{}>; rel=preconnect".format(second).encode()),
        ],
        end_headers=True)

    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "multiple-early-hints-preconnects.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
