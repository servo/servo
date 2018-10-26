def main(request, response):
    status = int(request.GET["status"])
    module = b"\0asm\1\0\0\0"
    return status, [("Content-Type", "application/wasm")], module
