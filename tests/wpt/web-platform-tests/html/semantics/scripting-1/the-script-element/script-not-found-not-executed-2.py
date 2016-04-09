def main(request, response):
    headers = [("Content-Type", "text/javascript")]
    body = "test2_token = \"script executed\";"
    return 200, headers, body
