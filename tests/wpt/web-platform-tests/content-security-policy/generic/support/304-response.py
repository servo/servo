def main(request, response):
    if request.headers.get("If-None-Match"):
        # we are now receing the second request, we will send back a different CSP
        # with the 304 response
        response.status = 304
        headers = [("Content-Type", "text/html"),
                   ("Content-Security-Policy", "script-src 'nonce-def' 'sha256-IIB78ZS1RMMrAWpsLg/RrDbVPhI14rKm3sFOeKPYulw=';"),
                   ("Cache-Control", "private, max-age=0, must-revalidate"),
                   ("ETag", "123456")]
        return headers, ""
    else:
        headers = [("Content-Type", "text/html"),
                   ("Content-Security-Policy", "script-src 'nonce-abc' 'sha256-IIB78ZS1RMMrAWpsLg/RrDbVPhI14rKm3sFOeKPYulw=';"),
                   ("Cache-Control", "private, max-age=0, must-revalidate"),
                   ("Etag", "123456")]
        return headers, '''
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
