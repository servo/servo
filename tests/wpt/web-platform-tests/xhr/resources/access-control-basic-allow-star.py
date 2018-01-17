#!/usr/bin/env python
def main(request, response):
    response.headers.set("Content-Type", "text/plain")
    response.headers.set("Access-Control-Allow-Origin", "*")

    response.content = "PASS: Cross-domain access allowed."
