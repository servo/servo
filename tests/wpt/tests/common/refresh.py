def main(request, response):
    """
    Respond with a blank HTML document and a `Refresh` header which describes
    an immediate redirect to the URL specified by the requests `location` query
    string parameter
    """
    headers = [
        (b'Content-Type', b'text/html'),
        (b'Refresh', b'0; URL=' + request.GET.first(b'location'))
    ]
    return (200, headers, b'')
