def main(request, response):
    headers = [("Content-Encoding", "gzip")]
    return headers, "not actually gzip"
