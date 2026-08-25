def main(request, response):
    if b"full" in request.GET:
        return request.url
    else:
        return request.request_path
