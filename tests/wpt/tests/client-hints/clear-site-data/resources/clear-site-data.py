"""
Step 5 (/client-hints/clear-site-data/clear-site-data-{}.https.html)
"""
def main(request, response):
    content = u'''
        <script>
            window.onload = function() {
                window.location.href = "/client-hints/clear-site-data/resources/check-client-hints.py";
            };
        </script>'''
    headers = [(b"Content-Type", b"text/html"), (b"Clear-Site-Data", b'"%s"' % (request.GET.first(b"target", b"*")))]
    return 200, headers, content
