def main(request, response):
    """Simple handler that causes redirection but also adds the
    "Access-Control-Allow-Origin: *" header.

    The request should typically have two query parameters:
    status - The status to use for the redirection. Defaults to 302.
    location - The resource to redirect to.
    """
    status = 302
    if b"status" in request.GET:
        try:
            status = int(request.GET.first(b"status"))
        except ValueError:
            pass

    response.status = status

    location = request.GET.first(b"location")

    response.headers.set(b"Location", location)
    response.headers.append(b"Access-Control-Allow-Origin", b"*")
