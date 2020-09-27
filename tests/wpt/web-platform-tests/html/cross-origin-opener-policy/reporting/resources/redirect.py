def main(request, response):
    response.status = 302
    response.headers.set(b"Location", request.url[request.url.find('?')+1:])
