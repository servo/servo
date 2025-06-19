import json

def create_echo_response(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    headers = {}
    for header in request.headers:
        key = header.decode('utf-8')
        value = request.headers.get(header).decode('utf-8')
        headers[key] = value
    result = json.dumps(headers)
    # If there is a callback, treat it as JSONP and wrap the result in the provided callback
    if b'callback' in request.GET:
        callback = request.GET.first(b"callback").decode('utf-8')
        result = callback + '(' + result + ');'

    return result
