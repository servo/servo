import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    message = {}

    header = request.headers.get(b"Test-Header-Injection");
    message[u'test_header_injection'] = isomorphic_decode(header) if header else None

    header = request.headers.get(b"Sec-Required-CSP");
    message[u'required_csp'] = isomorphic_decode(header) if header else None

    second_level_iframe_code = u""
    if b"include_second_level_iframe" in request.GET:
       if b"second_level_iframe_csp" in request.GET and request.GET[b"second_level_iframe_csp"] != b"":
         second_level_iframe_code = u'''<script>
            var i2 = document.createElement('iframe');
            i2.src = 'echo-required-csp.py';
            i2.csp = "{0}";
            document.body.appendChild(i2);
            </script>'''.format(isomorphic_decode(request.GET[b"second_level_iframe_csp"]))
       else:
         second_level_iframe_code = u'''<script>
            var i2 = document.createElement('iframe');
            i2.src = 'echo-required-csp.py';
            document.body.appendChild(i2);
            </script>'''

    return [(b"Content-Type", b"text/html"), (b"Allow-CSP-From", b"*")], u'''
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
