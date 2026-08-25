from wptserve.utils import isomorphic_encode

def main(request, response):
    values = request.GET.get_list(b"value")
    content = request.GET.first(b"content", b"<b>hi</b>\n")
    output = b"HTTP/1.1 200 OK\r\n"
    output += b"X-Content-Type-Options: nosniff\r\n"
    if b"single_header" in request.GET:
        output += b"Content-Type: " + b",".join(values) + b"\r\n"
    else:
        for value in values:
            output += b"Content-Type: " + value + b"\r\n"
    output += b"Content-Length: " + isomorphic_encode(str(len(content))) + b"\r\n"
    output += b"Connection: close\r\n"
    output += b"\r\n"
    output += content
    response.writer.write(output)
    response.close_connection = True
