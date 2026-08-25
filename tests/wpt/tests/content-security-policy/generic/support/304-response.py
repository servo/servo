def main(request, response):
    if request.headers.get(b"If-None-Match"):
        # we are now receing the second request, we will send back a different CSP
        # with the 304 response
        response.status = 304
        headers = [(b"Content-Type", b"text/html"),
                   (b"Content-Security-Policy", b"script-src 'nonce-def' 'sha256-IIB78ZS1RMMrAWpsLg/RrDbVPhI14rKm3sFOeKPYulw='"),
                   (b"Cache-Control", b"private, max-age=0, must-revalidate"),
                   (b"ETag", b"123456")]
        return headers, u""
    else:
        headers = [(b"Content-Type", b"text/html"),
                   (b"Content-Security-Policy", b"script-src 'nonce-abc' 'sha256-IIB78ZS1RMMrAWpsLg/RrDbVPhI14rKm3sFOeKPYulw='"),
                   (b"Cache-Control", b"private, max-age=0, must-revalidate"),
                   (b"Etag", b"123456")]
        return headers, u'''
<!DOCTYPE html>
<html>
<head>
    <script>
        window.addEventListener("securitypolicyviolation", function(e) {
            top.postMessage(e.originalPolicy, '*');
        });
    </script>
    <script nonce="abc">
        top.postMessage('abc_executed', '*');
    </script>
    <script nonce="def">
        top.postMessage('def_executed', '*');
    </script>
</head>
</html>
'''
