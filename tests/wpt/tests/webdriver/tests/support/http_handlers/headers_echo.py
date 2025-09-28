import json


def main(request, response):
    """Simple handler that returns a response with Cache-Control max-age=3600.
    """

    response.headers.set(b"Content-Type", b"text/plain")

    headers_dict = {}
    for key, value in request.headers.items():
        # Decode the key once
        decoded_key = key.decode("utf-8")

        # The value can be a single byte string or a list of byte strings
        # if the same header is present multiple times.
        if isinstance(value, list):
            # If it's a list, decode each item in it.
            headers_dict[decoded_key] = [v.decode("utf-8") for v in value]
        else:
            # Otherwise, just decode the single value.
            headers_dict[decoded_key] = value.decode("utf-8")

    response.content = json.dumps({
        "headers": headers_dict
    })
