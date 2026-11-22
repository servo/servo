import json


def main(request, response):
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")

    # The issuer identifier is a bare host name: requests to it reach this
    # server through the PAC file the tests install.
    host = request.url_parts.hostname

    return json.dumps({
        "Status": 0,
        "Answer": [{
            "type": 16,
            "data": f"iss={host}"
        }]
    })
