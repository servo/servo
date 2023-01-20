from time import sleep

def handle_headers(frame, request, response):
    delay = int(request.GET.first(b"delay")) / 1000
    sleep(delay)
    response.writer.write_raw_header_frame(headers=[(b":status", b"100")], end_headers=True)
    sleep(delay)

    if b"with103" in request.GET:
        response.writer.write_raw_header_frame(headers=[(b":status", b"103")], end_headers=True)
        sleep(delay)

    response.status = 200

    if b"tao" in request.GET:
        response.headers[b"timing-allow-origin"] = "*"

    response.headers[b"content-type"] = "text/plain"
    response.headers[b"access-control-allow-origin"] = "*"
    response.write_status_headers()

def main(request, response):
    response.writer.write_data(item="Hello World", last=True)
