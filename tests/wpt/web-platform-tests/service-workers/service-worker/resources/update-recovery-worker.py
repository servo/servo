def main(request, response):
    # Set mode to 'init' for initial fetch.
    mode = 'init'
    if 'update-recovery-mode' in request.cookies:
        mode = request.cookies['update-recovery-mode'].value

    # no-cache itself to ensure the user agent finds a new version for each update.
    headers = [('Cache-Control', 'no-cache, must-revalidate'),
               ('Pragma', 'no-cache')]

    extra_body = ''

    if mode == 'init':
        # Install a bad service worker that will break the controlled
        # document navigation.
        response.set_cookie('update-recovery-mode', 'bad')
        extra_body = "addEventListener('fetch', function(e) { e.respondWith(Promise.reject()); });"
    elif mode == 'bad':
        # When the update tries to pull the script again, update to
        # a worker service worker that does not break document
        # navigation.  Serve the same script from then on.
        response.delete_cookie('update-recovery-mode')

    headers.append(('Content-Type', 'application/javascript'))
    return headers, '%s' % (extra_body)
