import json


def main(request, response):
    """Handler that returns the request body and Content-Length header as JSON."""

    response.headers.set(b"Content-Type", b"text/plain")

    content_length = request.headers.get(b"Content-Length")
    response.content = json.dumps(
        {
            "body": request.body.decode("utf-8"),
            "contentLength": content_length.decode("utf-8")
            if content_length is not None
            else None,
        }
    )
