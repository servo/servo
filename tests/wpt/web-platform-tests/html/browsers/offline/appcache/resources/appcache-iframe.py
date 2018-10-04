script = '''
<script>
function fetchCachedFileTest() {
  return fetch('appcache-data.py?type=cached')
    .then(res => res.text(),
          _ => { throw new Error('Failed to fetch cached file'); })
    .then(text => {
      if (text != 'cached') {
        throw new Error('cached file missmatch');
      }
    });
}

function fetchNotInCacheFileTest() {
  return fetch('appcache-data.py?type=not-in-cache')
    .then(_ => { throw new Error('Fetching not-in-cache file must fail'); },
          _ => {});
}

function fetchFallbackFileTest() {
  return fetch('appcache-data.py?type=fallingback')
    .then(res => res.text(),
          _ => { throw new Error('Failed to fetch fallingback file'); })
    .then(text => {
      if (text != 'fallbacked') {
        throw new Error('fallbacked file miss match');
      }
    });
}

fetchCachedFileTest()
  .then(fetchNotInCacheFileTest)
  .then(fetchFallbackFileTest)
  .then(_ => window.parent.postMessage('Done: %s'),
        error => window.parent.postMessage(error.toString()));
</script>
'''

def main(request, response):
    type = request.GET['type']
    if request.GET['type'] == 'fallingback':
        return 404, [('Content-Type', 'text/plain')], "Page not found"
    return [('Content-Type', 'text/html')], script % type
