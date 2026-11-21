def main(request, response):
    content = 'span::before { content: "\u00e9"; }'.encode("utf-8")
    if b"bom" in request.GET:
        content = b"\xef\xbb\xbf" + content
    return 200, [(b"Content-Type", b"text/css; charset=windows-1252")], content
