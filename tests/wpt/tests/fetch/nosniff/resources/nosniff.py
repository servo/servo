def main(request, response):
    response.add_required_headers = False
    output =  b"HTTP/1.1 220 YOU HAVE NO POWER HERE\r\n"
    output += b"Content-Length: 22\r\n"
    output += b"Connection: close\r\n"
    output += b"Content-Type: x/x\r\n"
    output += request.GET.first(b"nosniff") + b"\r\n"
    output += b"\r\n"
    output += b"// nothing to see here"
    response.writer.write(output)
    response.close_connection = True
