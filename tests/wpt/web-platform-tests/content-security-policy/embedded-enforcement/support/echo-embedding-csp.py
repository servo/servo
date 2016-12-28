import json
def main(request, response):
    header = request.headers.get("Embedding-CSP");
    message = {}
    message['embedding_csp'] = header if header else None
    return [("Content-Type", "text/html"), ("Allow-CSP-From", "*")], '''
<!DOCTYPE html>
<html>
<head>
    <script>
      window.parent.postMessage({0}, '*');
    </script>
</head>
</html>
'''.format(json.dumps(message))
