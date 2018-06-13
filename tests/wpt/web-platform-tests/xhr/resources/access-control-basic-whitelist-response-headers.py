def main(request, response):
    headers = {
        # CORS-safelisted
        "content-type": "text/plain",
        "cache-control": "no cache",
        "content-language": "en",
        "expires": "Fri, 30 Oct 1998 14:19:41 GMT",
        "last-modified": "Tue, 15 Nov 1994 12:45:26 GMT",
        "pragma": "no-cache",

        # Non-CORS-safelisted
        "x-test": "foobar",

        "Access-Control-Allow-Origin": "*"
    }
    for header in headers:
        response.headers.set(header, headers[header])

    response.content = "PASS: Cross-domain access allowed."
