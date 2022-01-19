import json
def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    if b"allow_csp_from" in request.GET:
        headers.append((b"Allow-CSP-From", request.GET[b"allow_csp_from"]))
    message = request.GET[b"id"]
    return headers, b'''
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
    <script nonce="123">
        let img = document.createElement('img');
        img.src = "../../support/pass.png";
        img.onload = function() { window.top.postMessage("img loaded", '*'); }
        document.body.appendChild(img);
    </script>
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
