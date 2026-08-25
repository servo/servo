from time import sleep

def main(request, response):
    delay = int(request.GET.first(b"delay")) / 1000

    # TODO: make this exported from ResponseWriter
    handler = response.writer._handler
    if b"with100" in request.GET:
        sleep(delay)
        handler.send_response(100)
        handler.end_headers()

    if b"with103" in request.GET:
        sleep(delay)
        handler.send_response(103)
        handler.send_header("Link", "<resources/empty.js>;rel=preload;as=script")
        handler.end_headers()

    sleep(delay)

    handler.send_response(200)

    if b"tao" in request.GET:
        handler.send_header("timing-allow-origin", "*")

    handler.send_header("content-type", "text/plain")
    handler.send_header("access-control-allow-origin", "*")
    handler.end_headers()
    handler.wfile.write(bytes("Hello World", "utf8"))
