import time

def main(request, response):
    delay = float(request.GET.first(b"ms", 500)) / 1E3
    count = int(request.GET.first(b"count", 50))
    # Read request body
    request.body
    time.sleep(delay)
    if not b"notype" in request.GET:
        response.headers.set(b"Content-type", b"text/plain")
    response.write_status_headers()
    time.sleep(delay)
    for i in range(count):
        response.writer.write_content(b"TEST_TRICKLE\n")
        time.sleep(delay)
