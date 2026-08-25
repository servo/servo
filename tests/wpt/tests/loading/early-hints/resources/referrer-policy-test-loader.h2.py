import os
import time


def handle_headers(frame, request, response):
    headers = []
    referrer_policy = request.GET.first(b"referrer-policy")
    headers.append((b"referrer-policy", referrer_policy))

    preload_url = request.GET.first(b"same-origin-preload-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(preload_url)
    headers.append((b"link", link_header_value))
    preload_url = request.GET.first(b"cross-origin-preload-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(preload_url)
    headers.append((b"link", link_header_value))

    # Send a 103 response.
    early_hints = [(b":status", b"103")]
    for header in headers:
        early_hints.append(header)
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Simulate the response generation is taking time.
    time.sleep(0.2)

    response.status = 200
    response.headers["content-type"] = "text/html"
    for (name, value) in headers:
        response.headers[name] = value
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "referrer-policy-test.html")
    with open(file_path, "r") as f:
        test_content = f.read()
    response.writer.write_data(item=test_content, last=True)
