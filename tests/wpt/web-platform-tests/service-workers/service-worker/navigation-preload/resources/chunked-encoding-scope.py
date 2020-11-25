import time

def main(request, response):
    use_broken_body = b'use_broken_body' in request.GET

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"Content-type", b"text/html; charset=UTF-8")
    response.writer.write_header(b"Transfer-encoding", b"chunked")
    response.writer.end_headers()

    for idx in range(10):
        if use_broken_body:
            response.writer.write(u"%s\n%s\n" % (len(str(idx)), idx))
        else:
            response.writer.write(u"%s\r\n%s\r\n" % (len(str(idx)), idx))
        time.sleep(0.001)

    response.writer.write(u"0\r\n\r\n")
