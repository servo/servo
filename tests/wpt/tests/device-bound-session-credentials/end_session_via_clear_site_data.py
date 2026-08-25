import json

def main(request, response):
    types = request.body.decode('utf-8')
    if types == "":
        types = '"cookies"'
    return (200, [("Clear-Site-Data", types)], "")
