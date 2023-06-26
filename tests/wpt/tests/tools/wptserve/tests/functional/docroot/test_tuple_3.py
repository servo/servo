def main(request, response):
    return (202, "Giraffe"), [("Content-Type", "text/html"), ("X-Test", "PASS")], "PASS"
