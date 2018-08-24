import time

def main(request, response):
    # no-cache itself to ensure the user agent finds a new version for each update.
    headers = [('Cache-Control', 'no-cache, must-revalidate'),
               ('Pragma', 'no-cache')]
    content_type = 'application/javascript'

    headers.append(('Content-Type', content_type))

    body = '''
let promise = self.registration.update()
onmessage = (evt) => {
  promise.then(r => {
    evt.source.postMessage(self.registration === r ? 'PASS' : 'FAIL');
  });
};'''
    return headers, '/* %s %s */ %s' % (time.time(), time.clock(), body)
