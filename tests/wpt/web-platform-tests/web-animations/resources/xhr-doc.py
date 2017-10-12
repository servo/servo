def main(request, response):
    headers = [("Content-type", "text/html;charset=utf-8")]
    content = "<!doctype html><div id=test></div>"

    return headers, content
