def main(request, response):
    response.add_required_headers = False
    output =  "HTTP/1.1 221 ALL YOUR BASE BELONG TO H1\r\n"
    output += "Access-Control-Allow-Origin: *\r\n"
    output += "BB-8: hey\r\n"
    output += "Content-Language: mkay\r\n"
    output += request.GET.first("expose") + "\r\n"
    output += "\r\n"
    response.writer.write(output)
    response.close_connection = True
