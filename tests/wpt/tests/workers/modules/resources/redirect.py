def main(request, response):
    """Simple handler that causes redirection.
    This is placed here to stay within the same directory during redirects,
    to avoid issues like https://crbug.com/1136775.
    """
    response.status = 302
    location = request.GET.first(b"location")
    response.headers.set(b"Location", location)
