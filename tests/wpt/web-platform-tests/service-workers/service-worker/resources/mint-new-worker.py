import time

body = '''
onactivate = (e) => e.waitUntil(clients.claim());
var resolve_wait_until;
var wait_until = new Promise(resolve => {
    resolve_wait_until = resolve;
  });
onmessage = (e) => {
    if (e.data == 'wait')
      e.waitUntil(wait_until);
    if (e.data == 'go')
      resolve_wait_until();
  };'''

def main(request, response):
    headers = [('Cache-Control', 'no-cache, must-revalidate'),
               ('Pragma', 'no-cache'),
               ('Content-Type', 'application/javascript')]

    skipWaiting = ''
    if 'skip-waiting' in request.GET:
      skipWaiting = 'skipWaiting();'

    return headers, '/* %s %s */ %s %s' % (time.time(), time.clock(), skipWaiting, body)
