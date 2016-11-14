import time

def main(request, response):
    delay = float(request.GET.first("ms", 1000)) / 1E3
    count = int(request.GET.first("count", 50))
    time.sleep(delay)
    response.headers.set("Transfer-Encoding", "chunked")
    response.write_status_headers()
    time.sleep(delay);
    for i in xrange(count):
        response.writer.write_content("a\r\nTEST_CHUNK\r\n")
        time.sleep(delay)
    response.writer.write_content("garbage")
