import random

import time

body = u'''
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
    headers = [(b'Cache-Control', b'no-cache, must-revalidate'),
               (b'Pragma', b'no-cache'),
               (b'Content-Type', b'application/javascript')]

    skipWaiting = u''
    if b'skip-waiting' in request.GET:
        skipWaiting = u'skipWaiting();'

    return headers, u'/* %s %s */ %s %s' % (time.time(), random.random(), skipWaiting, body)
