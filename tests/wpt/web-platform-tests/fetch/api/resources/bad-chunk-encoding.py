import time

def main(request, response):
    delay = float(request.GET.first(b"ms", 1000)) / 1E3
    count = int(request.GET.first(b"count", 50))
    time.sleep(delay)
    response.headers.set(b"Transfer-Encoding", b"chunked")
    response.write_status_headers()
    time.sleep(delay)
    for i in range(count):
        response.writer.write_content(b"a\r\nTEST_CHUNK\r\n")
        time.sleep(delay)
    response.writer.write_content(b"garbage")
