def main(request, response):
    output = '\x1b\x03)\x00\xa4\xcc\xde\xe2\xb3 vA\x00\x0c'
    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "br"),
               ("Content-Length", len(output))]

    return headers, output
