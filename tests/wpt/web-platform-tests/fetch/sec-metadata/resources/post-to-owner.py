import json

def main(request, response):
    headers = [("Content-Type", "text/html")]

    body = """
        <!DOCTYPE html>
        <script>
            var data = %s;
            if (window.opener)
                window.opener.postMessage(data, "*");
            if (window.top != window)
                window.top.postMessage(data, "*");
        </script>
    """ % json.dumps({
        "dest": request.headers.get("sec-fetch-dest", ""),
        "mode": request.headers.get("sec-fetch-mode", ""),
        "site": request.headers.get("sec-fetch-site", ""),
        "user": request.headers.get("sec-fetch-user", ""),
        })
    return headers, body
