def main(request, response):
    headers = [("Content-type", request.GET.first("mime"))]
    content = "console.log('Script loaded')"
    return 200, headers, content
