def main(request, response):
    referrer = request.headers.get("referer", "")
    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Origin", "*")];
    return (200, response_headers,
            "export const referrer = '" + referrer + "';")
