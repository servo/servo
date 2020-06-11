def main(request, response):
    ua = request.headers.get(b'sec-ch-ua', b'')
    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(b"Accept-CH", b"UA")
    response.headers.set(b"Accept-CH-Lifetime", b"10")
    response.content = b'''
<script>
  window.opener.postMessage({ header: '%s' }, "*");
</script>
Sec-CH-UA: %s
''' % (ua, ua)
