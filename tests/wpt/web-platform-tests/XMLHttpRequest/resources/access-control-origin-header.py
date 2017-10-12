#!/usr/bin/env python
def main(request, response):
    response.headers.set("Content-Type", "text/plain");
    response.headers.set("Cache-Control", "no-cache, no-store");
    response.headers.set("Access-Control-Allow-External", "true");
    response.headers.set("Access-Control-Allow-Origin", "*");

    response.content = "PASS: Cross-domain access allowed.\n"
    response.content += "HTTP_ORIGIN: " + request.headers.get("origin");
