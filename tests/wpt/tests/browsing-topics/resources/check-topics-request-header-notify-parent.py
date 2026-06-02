def main(request, response):
    """
    Returns an HTML response that notifies its parent frame the topics header
    via postMessage
    """

    topics_header = request.headers.get(b"sec-browsing-topics", b"NO_TOPICS_HEADER")

    headers = [(b"Content-Type", b"text/html"),
               (b"Access-Control-Allow-Origin", b"*")]
    content = b'''
<script>
  let parentOrOpener = window.opener || window.parent;
  parentOrOpener.postMessage({ topicsHeader: '%s'}, "*");
</script>
''' % (topics_header)

    return 200, headers, content
