def main(request, response):
    headers = [("Content-type", "text/html;charset=utf-8")]
    content = chr(0xff)

    return headers, content
