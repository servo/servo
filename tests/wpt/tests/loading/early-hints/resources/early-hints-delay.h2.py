import time

def handle_headers(frame, request, response):
    early_hints = [
        (b":status", b"103"),
        (b"link", b"</empty.js>; rel=preload; as=script"),
    ]

    time.sleep(int(request.GET.first(b"delay1")) / 1000)
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    time.sleep(int(request.GET.first(b"delay2")) / 1000)
    response.status = 200
    response.headers[b"content-type"] = "text/html"
    response.write_status_headers()


def main(request, response):
    response.writer.write_data(item="Hello", last=True)
