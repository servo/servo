def main(request, response):
    headers = [("Content-Type", "text/javascript")]
    body = "test1_token = \"script executed\";"
    return 404, headers, body
