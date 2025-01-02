def main(request, response):
    """
    Simple handler that redirects to echo-critical-hint.py.
    """

    response.status = 302
    location = request.GET.first(b"location")
    response.headers.set(b"Location", location)
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    if(request.GET.first(b"critical", None) is not None):
        hints = b"sec-ch-dpr,dpr"
        response.headers.append(b"Accept-CH", hints)
        response.headers.append(b"Critical-CH", hints)

