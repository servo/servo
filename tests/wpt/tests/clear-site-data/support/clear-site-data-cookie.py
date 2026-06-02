"""
Step 2/3 (/clear-site-data/set-cookie-{}-clear-{}.https.html)
"""
def main(request, response):
    headers = [(b"Content-Type", b"text/html")]
    clear_site_data_header = (b"Clear-Site-Data", b"\"" + request.GET.first(b"target", b"*") + b"\"")
    set_cookie_header = (b"Set-Cookie", b"testSetWithClear=true")
    if request.GET.first(b"location") == b"after":
        headers = headers + [clear_site_data_header, set_cookie_header]
    else:
        headers = headers + [set_cookie_header, clear_site_data_header]
    content = u'''
        <script>
            window.opener.postMessage(document.cookie , "*");
        </script>'''
    return 200, headers, content
