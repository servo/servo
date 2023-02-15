def main(request, response):
    content = request.GET.first(b"content")
    length = request.GET.first(b"length").decode("ascii")
    response.add_required_headers = False

    output =  b"HTTP/1.1 200 OK\r\n"
    output += b"Content-Type: text/plain;charset=UTF-8\r\n"
    output += b"Connection: close\r\n"
    if length == b"auto" :
        output += b"Content-Length:"
        output += "{0}".format(len(content)).encode("ascii")
        output += b"\r\n"
    elif length != b"none" :
        output += b"Content-Length:"
        output += "{0}".format(length).encode("ascii")
        output += b"\r\n"
    output += b"\r\n"
    output += content
    response.writer.write(output)
    response.close_connection = True
