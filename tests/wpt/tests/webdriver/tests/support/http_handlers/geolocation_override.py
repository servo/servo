import json


def main(request, response):
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Cache-Control", b"no-cache")
    response.content = json.dumps(
        {
            "status": "OK",
            "location": {
                "lat": 37.41857,
                "lng": -122.08769,
            },
            "accuracy": 42,
        }
    ).encode("utf-8")
