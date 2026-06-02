def main(request, response):
    topics_header = request.headers.get(b"sec-browsing-topics", b"NO_TOPICS_HEADER")
    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.content = topics_header
