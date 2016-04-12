def main(request, response):
    if "url" in request.GET:
        headers = [("Location", request.GET["url"])]
        return 302, headers, ''

    return [], '''
<!DOCTYPE html>
<script>
  window.parent.postMessage(
      {
        id: 'last_url',
        result: location.href
      }, '*');
</script>
'''
