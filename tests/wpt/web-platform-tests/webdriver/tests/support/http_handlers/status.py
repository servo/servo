def main(request, response):
    """Simple handler that returns a response with a custom status.

    The request expects a "status" query parameter, which should be a number.
    If no status is provided, status 200 will be used.
    """
    status = 200
    if b"status" in request.GET:
        try:
            status = int(request.GET.first(b"status"))
        except ValueError:
            pass

    response.status = status
    response.headers.set(b"Content-Type", "text/plain")
    response.content = "HTTP Response Status"
