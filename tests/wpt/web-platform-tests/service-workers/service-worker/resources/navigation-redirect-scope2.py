def main(request, response):
    if "url" in request.GET:
        headers = [("Location", request.GET["url"])]
        return 302, headers, ''

    status = 200

    if "noLocationRedirect" in request.GET:
        status = 302

    return status, [], '''
<!DOCTYPE html>
<script>
  window.parent.postMessage(
      {
        id: 'last_url',
        result: location.href
      }, '*');
</script>
'''
