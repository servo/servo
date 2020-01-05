def main(request, response):
    content = ""
    if "my-custom-header" in request.GET:
        val = request.GET.first("my-custom-header")
        response.headers.set("My-Custom-Header", val)
    return content
