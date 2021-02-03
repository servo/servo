def main(request, response):
    output = b"HTTP/1.1 "
    output += request.GET.first(b"input")
    output += b"\nheader-parsing: is sad\n"
    response.writer.write(output)
    response.close_connection = True
