def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    body = u"test1_token = \"script executed\";"
    return 404, headers, body
