import json


def main(request, response):
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    host = request.url_parts.hostname
    port = request.url_parts.port
    issuer = f"{host}:{port}" if port else host

    return json.dumps({
        "Status": 0,
        "Answer": [{
            "type": 16,
            "data": f"iss={issuer}"
        }]
    })
