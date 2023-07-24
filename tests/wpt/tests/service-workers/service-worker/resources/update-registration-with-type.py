def classic_script():
    return b"""
      importScripts('./imported-classic-script.js');
      self.onmessage = e => {
        e.source.postMessage(imported);
      };
      """

def module_script():
    return b"""
      import * as module from './imported-module-script.js';
      self.onmessage = e => {
        e.source.postMessage(module.imported);
      };
      """

# Returns the classic script for a first request and
# returns the module script for second and subsequent requests.
def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Pragma', b'no-store'),
               (b'Cache-Control', b'no-store')]

    classic_first = request.GET[b'classic_first']
    key = request.GET[b'key']
    requested_once = request.server.stash.take(key)
    if requested_once is None:
        request.server.stash.put(key, True)
        body = classic_script() if classic_first == b'1' else module_script()
    else:
        body = module_script() if classic_first == b'1' else classic_script()

    return 200, headers, body
