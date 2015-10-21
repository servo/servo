def main(request, response):
    headers = [("Content-type", "text/html;charset=utf-8")]
    content = "<img>foo"

    return headers, content
