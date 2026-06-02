def main(request, response):
    return [("Content-Type", "text/html"), ("X-Test", "PASS")], "PASS"
