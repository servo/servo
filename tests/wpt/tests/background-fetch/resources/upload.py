# Simply returns the request body to check if the upload succeeded.
def main(request, response):
    return 200, [(b"Content-Type", request.headers[b'content-type'])], request.body
