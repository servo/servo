def main(request, response):
    """Redirect handler that optionally sets a Timing-Allow-Origin header.

    Query parameters:
      status   - The status code to use for the redirection. Defaults to 302.
      location - The (percent-encoded) resource to redirect to.
      tao      - The value to send in the Timing-Allow-Origin response header. If
                 absent, no Timing-Allow-Origin header is sent (i.e. the redirect
                 does not opt in).
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
    if b"tao" in request.GET:
        response.headers.set(b"Timing-Allow-Origin", request.GET.first(b"tao"))
