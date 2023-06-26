def main(request, response):
    ua = request.headers.get(b'Sec-CH-UA', b'')
    response.headers.set(b"Content-Type", b"text/html")
    response.content = b'''
<script>
  window.opener.postMessage({ header: '%s' }, "*");
</script>
Sec-CH-UA: %s
''' % (ua, ua)
