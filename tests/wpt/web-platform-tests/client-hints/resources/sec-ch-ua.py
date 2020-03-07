def main(request, response):
    ua = request.headers.get('sec-ch-ua', '')
    response.headers.set("Content-Type", "text/html")
    response.headers.set("Accept-CH", "UA")
    response.headers.set("Accept-CH-Lifetime", "10")
    response.content = '''
<script>
  window.opener.postMessage({ header: '%s' }, "*");
</script>
Sec-CH-UA: %s
''' % (ua, ua)
