"""
Step 2/6 (/client-hints/clear-site-data/clear-site-data-{}.https.html)
Step 3/4 (/client-hints/clear-site-data/set-client-hints-{}-clear-{}.https.html)
"""
def main(request, response):
    if b"sec-ch-device-memory" in request.headers:
        result = u"HadDeviceMemory"
    else:
        result = u"MissingDeviceMemory"
    content = u'''
        <script>
            window.opener.postMessage("%s" , "*");
        </script>''' % (result)
    headers = [(b"Content-Type", b"text/html")]
    return 200, headers, content
