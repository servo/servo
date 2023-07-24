def main(request, response):
    bytes = bytearray(request.raw_input.read())
    bytes_string = b" ".join(b"%02x" % b for b in bytes)
    return (
        [(b"Content-Type", b"text/plain")],
        bytes_string
    )
