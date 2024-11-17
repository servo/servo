import json

def get_json(request, response):
    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.content = json.dumps([])
