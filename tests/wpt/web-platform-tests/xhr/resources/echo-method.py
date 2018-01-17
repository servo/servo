def main(request, response):
    response.send_body_for_head_request = True
    headers = [("Content-type", "text/plain")]
    content = request.method

    return headers, content
