import os
import time

def handle_headers(frame, request, response):
    resource_url = request.GET.first(b"resource-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(resource_url)
    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    # Sleep to simulate a slow generation of the final response.
    time.sleep(0.1)
    response.status = 200
    response.headers[b"content-type"] = "application/pdf"
    response.write_status_headers()


def main(request, response):
    current_dir = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(current_dir, "example.pdf")
    with open(file_path, "rb") as f:
        content = f.read()
    response.writer.write_data(item=content, last=True)
