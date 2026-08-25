def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=utf-8")]
    content = b"<!DOCTYPE html><div></div>"

    return headers, content
