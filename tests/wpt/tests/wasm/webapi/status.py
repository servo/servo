def main(request, response):
    status = int(request.GET[b"status"])
    module = b"\0asm\1\0\0\0"
    return status, [(b"Content-Type", b"application/wasm")], module
