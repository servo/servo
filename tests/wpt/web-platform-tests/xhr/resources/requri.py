def main(request, response):
    if "full" in request.GET:
        return request.url
    else:
        return request.request_path
