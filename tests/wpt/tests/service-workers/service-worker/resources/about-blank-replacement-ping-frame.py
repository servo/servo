def main(request, response):
  if b'nested' in request.GET:
    return (
      [(b'Content-Type', b'text/html')],
      b'failed: nested frame was not intercepted by the service worker'
    )

  return ([(b'Content-Type', b'text/html')], b"""
<!doctype html>
<html>
<body>
<script>
function nestedLoaded() {
  parent.postMessage({ type: 'NESTED_LOADED' }, '*');
}
</script>
<iframe src="?nested=true&amp;ping=true" id="nested" onload="nestedLoaded()"></iframe>
<script>
// Helper routine to make it slightly easier for our parent to find
// the nested frame.
function nested() {
  return document.getElementById('nested').contentWindow;
}

// This modifies the nested iframe immediately and does not wait for it to
// load.  This effectively modifies the global for the initial about:blank
// document.  Any modifications made here should be preserved after the
// frame loads because the global should be re-used.
let win = nested();
if (win.location.href !== 'about:blank') {
  parent.postMessage({
    type: 'NESTED_LOADED',
    result: 'failed: nested iframe does not have an initial about:blank URL'
  }, '*');
} else {
  win.navigator.serviceWorker.addEventListener('message', evt => {
    if (evt.data.type === 'PING') {
      evt.source.postMessage({
        type: 'PONG',
        location: win.location.toString()
      });
    }
  });
  win.navigator.serviceWorker.startMessages();
}
</script>
</body>
</html>
""")
