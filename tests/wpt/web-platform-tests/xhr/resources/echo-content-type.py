def main(request, response):
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Connection", b"close")
    response.status = 200
    response.content = request.headers.get(b"Content-Type")
    response.close_connection = True
