def main(request, response):
    bytes = bytearray(request.raw_input.read())
    bytes_string = " ".join("%02x" % b for b in bytes)
    return (
        [("Content-Type", "text/plain")],
        bytes_string
    )
