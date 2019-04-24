def main(request, response):
    time = request.url_parts.query if request.url_parts.query else '0'
    return 200, [('Refresh', time), ('Content-Type', "text/html")], ''
