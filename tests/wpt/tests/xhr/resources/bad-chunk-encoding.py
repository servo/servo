import time

def main(request, response):
    delay = 0.1
    count = 5
    time.sleep(delay)
    response.headers.set(b"Transfer-Encoding", b"chunked")
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"X-Content-Type-Options", b"nosniff")
    response.headers.set(b"Connection", b"close")
    response.close_connection = True
    response.write_status_headers()
    time.sleep(delay)
    for i in range(count):
        response.writer.write_content(b"a\r\nTEST_CHUNK\r\n")
        time.sleep(delay)
    response.writer.write_content(b"garbage")
