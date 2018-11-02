def main(request, response):
    response.add_required_headers = False
    output =  "HTTP/1.1 220 YOU HAVE NO POWER HERE\r\n"
    output += "Content-Length: 22\r\n"
    output += "Content-Type: x/x\r\n"
    output += request.GET.first("nosniff") + "\r\n"
    output += "\r\n"
    output += "// nothing to see here"
    response.writer.write(output)
    response.close_connection = True
