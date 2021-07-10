#!/usr/bin/env python
def main(request, response):
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    response.content = b"PASS: Cross-domain access allowed."
