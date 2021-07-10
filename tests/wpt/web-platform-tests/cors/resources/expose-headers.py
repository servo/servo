def main(request, response):
    response.add_required_headers = False
    output =  b"HTTP/1.1 221 ALL YOUR BASE BELONG TO H1\r\n"
    output += b"Access-Control-Allow-Origin: *\r\n"
    output += b"BB-8: hey\r\n"
    output += b"Content-Language: mkay\r\n"
    output += request.GET.first(b"expose") + b"\r\n"
    output += b"\r\n"
    response.writer.write(output)
    response.close_connection = True
