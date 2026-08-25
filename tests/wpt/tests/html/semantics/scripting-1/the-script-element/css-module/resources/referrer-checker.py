def main(request, response):
    referrer = request.headers.get(b"referer", b"")
    response_headers = [(b"Content-Type", b"text/css"),
                        (b"Access-Control-Allow-Origin", b"*")]
    # Put the referrer in a CSS rule that can be read by the importer through CSSOM
    return (200, response_headers,
            b'.referrer { content: "' + referrer + b'" }')
