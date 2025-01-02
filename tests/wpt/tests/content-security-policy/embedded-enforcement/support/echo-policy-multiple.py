def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    if b"policy" in request.GET:
        headers.append((b"Content-Security-Policy", request.GET[b"policy"]))
    if b"policy2" in request.GET:
        headers.append((b"Content-Security-Policy", request.GET[b"policy2"]))
    if b"policy3" in request.GET:
        headers.append((b"Content-Security-Policy", request.GET[b"policy3"]))
    message = request.GET[b"id"]
    return headers, b'''
<!DOCTYPE html>
<html>
<head>
    <title>This page sets given CSP upon itself.</title>
</head>
<body>
    <script nonce="abc">
        var response = {};
        response["id"] = "%s";
        response["loaded"] = true;
        window.top.postMessage(response, '*');
    </script>
</body>
</html>
''' % (message)
