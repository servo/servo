# -*- coding: utf-8 -*-

def main(request, response):
    response.headers.set("Content-Type", "text/plain")
    response.headers.set("X-Custom-Header", "test")
    response.headers.set("Set-Cookie", "test")
    response.headers.set("Set-Cookie2", "test")
    response.headers.set("X-Custom-Header-Empty", "")
    response.headers.set("X-Custom-Header-Comma", "1")
    response.headers.append("X-Custom-Header-Comma", "2")
    response.headers.set("X-Custom-Header-Bytes", "…")
    return "TEST"
