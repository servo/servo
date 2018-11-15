def main(request, response):
    content = "<meta charset=utf-8>\n<script>document.write(document.characterSet)</script>"

    # This uses the following rather than
    #   response.headers.set("Content-Type", request.GET.first("type"));
    #   response.content = content
    # to work around https://github.com/web-platform-tests/wpt/issues/8372.

    response.add_required_headers = False
    output =  "HTTP/1.1 200 OK\r\n"
    output += "Content-Length: " + str(len(content)) + "\r\n"
    output += "Content-Type: " + request.GET.first("type") + "\r\n"
    output += "\r\n"
    output += content
    response.writer.write(output)
    response.close_connection = True
