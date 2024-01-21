from pathlib import Path

# Returns incrementer.wasm, with appropriate headers. Depending on query
# parameter, it can simulate a variety of network errors.
def main(request, response):
    error = request.GET.first(b"error", None)

    if error == b"close-connection":
        # Close connection without writing anything, to simulate a network
        # error. The write call is needed to avoid writing the default headers.
        response.writer.write("")
        response.close_connection = True
        return

    if error == b"http-error":
        response.status = (404, b"OK")
    else:
        response.status = (200, b"OK")

    if error == b"wrong-content-type":
        response.headers.set(b"Content-Type", b"application/javascript")
    elif error != b"no-content-type":
        response.headers.set(b"Content-Type", b"application/wasm")

    if error == b"bad-allow-fledge":
        response.headers.set(b"Ad-Auction-Allowed", b"sometimes")
    elif error == b"fledge-not-allowed":
        response.headers.set(b"Ad-Auction-Allowed", b"false")
    elif error != b"no-allow-fledge":
        response.headers.set(b"Ad-Auction-Allowed", b"true")

    if error == b"no-body":
        return b""

    if error == b"not-wasm":
        return b"This is not wasm"

    return (Path(__file__).parent.resolve() / "incrementer.wasm").read_bytes()
