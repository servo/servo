def main(request, response):
    headers = [("Content-type", "text/html;charset=windows-1252")]
    content = chr(0xff)

    return headers, content
