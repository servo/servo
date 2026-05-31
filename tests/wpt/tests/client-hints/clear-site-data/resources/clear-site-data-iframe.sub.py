"""
Step 5 (/client-hints/clear-site-data/clear-site-data-client-hints-third-party.https.html)
"""
def main(request, response):
    content = b'''
        <iframe src="https://{{hosts[][]}}:{{ports[https][0]}}/client-hints/clear-site-data/resources/clear-site-data.py?%s">
        </iframe>''' % request.GET.first(b"target", b"*")
    headers = [(b"Content-Type", b"text/html")]
    return 200, headers, content
