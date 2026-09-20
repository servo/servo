def main(request, response):
    response.headers.set(b"content-type", b"text/plain")
    for value in request.GET.get_list(b"value"):
        response.headers.append(b"content-disposition", value)
    return b"Test.\n"
