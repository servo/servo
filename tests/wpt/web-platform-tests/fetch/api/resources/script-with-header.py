def main(request, response):
    headers = [("Content-type", request.GET.first("mime"))]
    if "content" in request.GET and request.GET.first("content") == "empty":
        content = ''
    else:
        content = "console.log('Script loaded')"
    return 200, headers, content
