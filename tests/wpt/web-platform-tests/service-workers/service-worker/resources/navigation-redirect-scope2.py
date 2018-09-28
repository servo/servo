def main(request, response):
    if "url" in request.GET:
        headers = [("Location", request.GET["url"])]
        return 302, headers, ''

    status = 200

    if "noLocationRedirect" in request.GET:
        status = 302

    return status, [("content-type", "text/html")], '''
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
