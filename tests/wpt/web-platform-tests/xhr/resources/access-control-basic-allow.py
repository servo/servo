#!/usr/bin/env python
def main(request, response):
    response.headers.set("Content-Type", "text/plain")
    response.headers.set("Access-Control-Allow-Credentials", "true")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))

    response.content = "PASS: Cross-domain access allowed."
