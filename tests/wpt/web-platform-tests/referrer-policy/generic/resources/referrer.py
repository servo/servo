def main(request, response):
    response_headers = [(b"Access-Control-Allow-Origin", b"*")]
    body = """
    <p id=referrer>%s</p>
    <script>
      const referrer_text = referrer.textContent;
      window.parent.postMessage(referrer_text, "*");
    </script>
    """ % request.headers.get("referer", "")
    return (200, response_headers, body)
