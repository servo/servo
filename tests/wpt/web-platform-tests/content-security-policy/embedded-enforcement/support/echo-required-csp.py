import json
def main(request, response):
    message = {}

    header = request.headers.get("Test-Header-Injection");
    message['test_header_injection'] = header if header else None

    header = request.headers.get("Sec-Required-CSP");
    message['required_csp'] = header if header else None

    second_level_iframe_code = ""
    if "include_second_level_iframe" in request.GET:
       if "second_level_iframe_csp" in request.GET and request.GET["second_level_iframe_csp"] <> "":
         second_level_iframe_code = '''<script>
            var i2 = document.createElement('iframe');
            i2.src = 'echo-required-csp.py';
            i2.csp = "{0}";
            document.body.appendChild(i2);
            </script>'''.format(request.GET["second_level_iframe_csp"])
       else:
         second_level_iframe_code = '''<script>
            var i2 = document.createElement('iframe');
            i2.src = 'echo-required-csp.py';
            document.body.appendChild(i2);
            </script>'''

    return [("Content-Type", "text/html"), ("Allow-CSP-From", "*")], '''
<!DOCTYPE html>
<html>
<head>
    <!--{2}-->
    <script>
      window.addEventListener('message', function(e) {{
        window.parent.postMessage(e.data, '*');
      }});

      window.parent.postMessage({0}, '*');
    </script>
</head>
<body>
{1}
</body>
</html>
'''.format(json.dumps(message), second_level_iframe_code, str(request.headers))
