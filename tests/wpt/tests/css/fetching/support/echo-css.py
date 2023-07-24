import sys
import json

def main(request, response):
    content = request.GET.first(b"content", None)
    response.headers.set(b"Content-Type", "text/css");
    return content
