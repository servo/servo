import time

def main(request, response):
    body = "hello\nworld\n\n"

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("Content-type", "text/html; charset=UTF-8")
    response.writer.write_header("Transfer-encoding", "chunked")
    response.writer.end_headers()

    for idx in range(10):
        response.writer.write("%s\r\n%s\r\n" % (len(str(idx)), idx))
        response.writer.flush()
        time.sleep(0.001)

    response.writer.write("0\r\n\r\n")
