def main(request, response):
  if 'nested' in request.GET:
    return (
      [('Content-Type', 'text/html')],
      'failed: nested frame was not intercepted by the service worker'
    )

  return ([('Content-Type', 'text/html')], """
<!doctype html>
<html>
<body>
<script>
function nestedLoaded() {
  parent.postMessage({ type: 'NESTED_LOADED' }, '*');
}

let popup = window.open('?nested=true');
popup.onload = nestedLoaded;

addEventListener('unload', evt => {
  popup.close();
}, { once: true });

// Helper routine to make it slightly easier for our parent to find
// the nested popup window.
function nested() {
  return popup;
}
</script>
</body>
</html>
""")
