def main(request, response):
    if b"url" in request.GET:
        headers = [(b"Location", request.GET[b"url"])]
        return 302, headers, b''

    status = 200

    if b"noLocationRedirect" in request.GET:
        status = 302

    return status, [(b"content-type", b"text/html")], b'''
<!DOCTYPE html>
<script>
onmessage = event => {
  window.parent.postMessage(
      {
        id: event.data.id,
        result: location.href
      }, '*');
};
</script>
'''
