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
</script>
<iframe src="?nested=true" id="nested" onload="nestedLoaded()"></iframe>
<script>
// Helper routine to make it slightly easier for our parent to find
// the nested frame.
function nested() {
  return document.getElementById('nested').contentWindow;
}

// NOTE: Make sure not to touch the iframe directly here.  We want to
//       test the case where the initial about:blank document is not
//       directly accessed before load.
</script>
</body>
</html>
""")
