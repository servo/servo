def main(request, response):
    headers = [
        ("Content-Type", "text/javascript"),
        ("Access-Control-Allow-Origin", request.headers.get("Origin")),
        ("Timing-Allow-Origin", request.headers.get("Origin")),
        ("Access-Control-Allow-Credentials", "true")
    ]

    return headers, "// Cross-origin module, nothing to see here"
