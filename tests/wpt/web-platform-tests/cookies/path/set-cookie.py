import json

# TODO: when given a cookie name and path it should return them in a Set-Cookie header

def main(request, response):
    headers = []
    values = []
    print "got req", dir(request)
    body = ""
    return headers, body
