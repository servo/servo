def main(request, response):
    values = request.GET.get_list("value")
    content = request.GET.first("content", "<b>hi</b>\n")
    output =  "HTTP/1.1 200 OK\r\n"
    output += "X-Content-Type-Options: nosniff\r\n"
    if "single_header" in request.GET:
        output += "Content-Type: " + ",".join(values) + "\r\n"
    else:
        for value in values:
            output += "Content-Type: " + value + "\r\n"
    output += "Content-Length: " + str(len(content)) + "\r\n"
    output += "\r\n"
    output += content
    response.writer.write(output)
    response.close_connection = True
