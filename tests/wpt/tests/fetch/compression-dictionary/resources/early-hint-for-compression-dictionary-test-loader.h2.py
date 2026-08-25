import os
import time

def _remove_relative_resources_prefix(path):
    if path.startswith("resources/"):
        return path[len("resources/"):]
    return path

def handle_headers(frame, request, response):
    # Send a 103 response.
    dictionary_url = request.GET.first(b"dictionary_url").decode()
    link_header_value = "<{}>; rel=\"compression-dictionary\"".format(dictionary_url).encode()
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Delay before sending the 200 response.
    time.sleep(0.2)
    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()

def main(request, response):
    test_path = _remove_relative_resources_prefix(
        request.GET[b"test_url"].decode("utf-8"))
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, test_path)
    test_content = open(file_path, "r").read()
    response.writer.write_data(item=test_content, last=True)
