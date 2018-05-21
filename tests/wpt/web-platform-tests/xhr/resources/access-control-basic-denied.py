def main(request, response):
    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Content-Type", "text/plain")

    response.text = "FAIL: Cross-domain access allowed."
