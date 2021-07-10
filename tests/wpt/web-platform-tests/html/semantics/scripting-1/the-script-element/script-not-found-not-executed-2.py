def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    body = u"test2_token = \"script executed\";"
    return 200, headers, body
