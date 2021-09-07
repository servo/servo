import time

def main(request, response):
    """Simple handler that causes redirection.

    The request should typically have two query parameters:
    status - The status to use for the redirection. Defaults to 302.
    location - The resource to redirect to.
    """
    status = 302
    delay = 2
    if b"status" in request.GET:
        try:
            status = int(request.GET.first(b"status"))
        except ValueError:
            pass

    if b"delay" in request.GET:
        try:
            delay = int(request.GET.first(b"delay"))
        except ValueError:
            pass

    response.status = status
    time.sleep(delay)

    location = request.GET.first(b"location")

    response.headers.set(b"Location", location)
