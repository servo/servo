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
    """ % json.dumps(request.headers.get("Sec-Metadata", ""))
    return headers, body
