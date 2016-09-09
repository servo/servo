import time

def main(request, response):
    chunk = "TEST_TRICKLE\n"
    delay = float(request.GET.first("ms", 500)) / 1E3
    count = int(request.GET.first("count", 50))
    if "specifylength" in request.GET:
        response.headers.set("Content-Length", count * len(chunk))
    time.sleep(delay)
    response.headers.set("Content-type", "text/plain")
    response.write_status_headers()
    time.sleep(delay);
    for i in xrange(count):
        response.writer.write_content(chunk)
        time.sleep(delay)
