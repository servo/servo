def main(request, response):
    headers = [("Content-Type", "text/html")]
    if "policy" in request.GET:
        headers.append(("Content-Security-Policy", request.GET["policy"]))
    if "policy2" in request.GET:
        headers.append(("Content-Security-Policy", request.GET["policy2"]))
    if "policy3" in request.GET:
        headers.append(("Content-Security-Policy", request.GET["policy3"]))
    message = request.GET["id"]
    return headers, '''
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
