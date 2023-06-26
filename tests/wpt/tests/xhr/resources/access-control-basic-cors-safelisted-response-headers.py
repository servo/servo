def main(request, response):
    headers = {
        # CORS-safelisted
        b"content-type": b"text/plain",
        b"cache-control": b"no cache",
        b"content-language": b"en",
        b"expires": b"Fri, 30 Oct 1998 14:19:41 GMT",
        b"last-modified": b"Tue, 15 Nov 1994 12:45:26 GMT",
        b"pragma": b"no-cache",

        # Non-CORS-safelisted
        b"x-test": b"foobar",

        b"Access-Control-Allow-Origin": b"*"
    }
    for header in headers:
        response.headers.set(header, headers[header])

    response.content = b"PASS: Cross-domain access allowed."
