def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=utf-8")]
    content = u"<!doctype html><div id=test></div>"

    return headers, content
