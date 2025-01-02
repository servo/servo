from wptserve.utils import isomorphic_encode

def main(request, response):
    content = b"<meta charset=utf-8>\n<script>document.write(document.characterSet)</script>"

    # This uses the following rather than
    #   response.headers.set("Content-Type", request.GET.first("type"));
    #   response.content = content
    # to work around https://github.com/web-platform-tests/wpt/issues/8372.

    response.add_required_headers = False
    output = b"HTTP/1.1 200 OK\r\n"
    output += b"Content-Length: " + isomorphic_encode(str(len(content))) + b"\r\n"
    output += b"Content-Type: " + request.GET.first(b"type") + b"\r\n"
    output += b"Connection: close\r\n"
    output += b"\r\n"
    output += content
    response.writer.write(output)
    response.close_connection = True
