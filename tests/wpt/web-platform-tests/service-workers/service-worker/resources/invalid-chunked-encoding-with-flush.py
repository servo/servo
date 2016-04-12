import time
def main(request, response):
    response.headers.set("Content-Type", "application/javascript")
    response.headers.set("Transfer-encoding", "chunked")
    response.write_status_headers()

    time.sleep(1)
    response.explicit_flush = True

    response.writer.write("XX\r\n\r\n")
    response.writer.flush()
