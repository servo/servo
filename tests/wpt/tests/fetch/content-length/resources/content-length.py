def main(request, response):
    response.add_required_headers = False
    output =  b"HTTP/1.1 200 OK\r\n"
    output += b"Content-Type: text/plain;charset=UTF-8\r\n"
    output += b"Connection: close\r\n"
    output += request.GET.first(b"length") + b"\r\n"
    output += b"\r\n"
    output += b"Fact: this is really forty-two bytes long."
    response.writer.write(output)
    response.close_connection = True
