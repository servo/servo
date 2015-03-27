def main(request, response):
    response.headers.update([('Transfer-Encoding', 'chunked'),
                             ('Content-Type', 'text/html'),
                             ('Connection', 'keep-alive')])
    response.write_status_headers()
    response.explicit_flush = True

    string = "W3C"
    for i in xrange(1000):
        response.writer.write("%s\r\n%s\r\n" % (len(string), string))
        response.writer.flush();

    response.writer.write("0\r\n\r\n")
    response.writer.flush();

