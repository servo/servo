def main(request, response):
    """
    postMessage with Viewport-Width and Sec-Ch-Viewport-Height headers
    """

    if b"sec-ch-viewport-width" in request.headers:
        width = request.headers["sec-ch-viewport-width"]
    else:
        width = b"FAIL"

    if b"sec-ch-viewport-height" in request.headers:
        height = request.headers["sec-ch-viewport-height"]
    else:
        height = b"FAIL"

    headers = [(b"Content-Type", b"text/html"),
               (b"Access-Control-Allow-Origin", b"*")]
    content = b'''
<script>
  let parentOrOpener = window.opener || window.parent;
  parentOrOpener.postMessage({ viewportWidth: '%s', viewportHeight: '%s' }, "*");
</script>
''' % (width, height)

    return 200, headers, content
