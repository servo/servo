import json
def main(request, response):
    headers = [("Content-Type", "text/html")]
    if "allow_csp_from" in request.GET:
        headers.append(("Allow-CSP-From", request.GET["allow_csp_from"]))
    message = request.GET["id"]
    return headers, '''
<!DOCTYPE html>
<html>
<head>
    <title>This page enforces embedder's policies</title>
    <script nonce="123">
        document.addEventListener("securitypolicyviolation", function(e) {
            var response = {};
            response["id"] = "%s";
            response["securitypolicyviolation"] = true;
            response["blockedURI"] = e.blockedURI;
            response["lineNumber"] = e.lineNumber;
            window.top.postMessage(response, '*');
        });
    </script>
</head>
<body>
    <style>
        body {
            background-color: maroon;
        }
    </style>
    <script nonce="abc"> 
        var response = {};
        response["id"] = "%s";
        response["loaded"] = true;
        window.top.postMessage(response, '*');
    </script>
</body>
</html>
''' % (message, message)
