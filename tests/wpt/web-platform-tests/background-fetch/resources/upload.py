# Simply returns the request body to check if the upload succeeded.
def main(request, response):
    return 200, [("Content-Type", request.headers['content-type'])], request.body
