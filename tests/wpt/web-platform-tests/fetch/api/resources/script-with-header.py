def main(request, response):
    headers = [(b"Content-type", request.GET.first(b"mime"))]
    if b"content" in request.GET and request.GET.first(b"content") == b"empty":
        content = b''
    else:
        content = b"console.log('Script loaded')"
    return 200, headers, content
