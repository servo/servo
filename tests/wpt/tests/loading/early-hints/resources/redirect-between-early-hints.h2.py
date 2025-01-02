import os


def _send_early_hints(preload, writer):
    link_header_value = "<{}>; rel=preload; as=script".format(preload)
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    writer.write_raw_header_frame(headers=early_hints, end_headers=True)


def handle_headers(frame, request, response):
    step = request.GET.first(b"test-step").decode()
    if step == "redirect":
        preload = request.GET.first(b"preload-before-redirect").decode()
        _send_early_hints(preload, response.writer)

        # Redirect to the final test page with parameters.
        params = []
        for key, values in request.GET.items():
            if key == b"test-step":
                params.append("test-step=final-response")
            else:
                params.append("{}={}".format(key.decode(), values[0].decode()))

        redirect_url = request.GET.first(b"redirect-url").decode()
        location = "{}?{}".format(redirect_url, "&".join(params))

        response.status = 302
        response.headers["location"] = location
        response.write_status_headers()
    elif step == "final-response":
        preload = request.GET.first(b"preload-after-redirect").decode()
        _send_early_hints(preload, response.writer)

        response.status = 200
        response.headers["content-type"] = "text/html"
        response.write_status_headers()
    else:
        raise Exception("Invalid step: {}".format(step))


def main(request, response):
    step = request.GET.first(b"test-step").decode()
    if step != "final-response":
        return

    final_test_page = request.GET.first(b"final-test-page").decode()
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, final_test_page)
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
