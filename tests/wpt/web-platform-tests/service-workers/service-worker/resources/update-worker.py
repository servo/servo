import time

def main(request, response):
    # Set mode to 'init' for initial fetch.
    mode = 'init'
    if 'mode' in request.cookies:
        mode = request.cookies['mode'].value

    # no-cache itself to ensure the user agent finds a new version for each update.
    headers = [('Cache-Control', 'no-cache, must-revalidate'),
               ('Pragma', 'no-cache')]

    content_type = ''
    extra_body = ''

    if mode == 'init':
        # Set a normal mimetype.
        # Set cookie value to 'normal' so the next fetch will work in 'normal' mode.
        content_type = 'application/javascript'
        response.set_cookie('mode', 'normal')
    elif mode == 'normal':
        # Set a normal mimetype.
        # Set cookie value to 'error' so the next fetch will work in 'error' mode.
        content_type = 'application/javascript'
        response.set_cookie('mode', 'error');
    elif mode == 'error':
        # Set a disallowed mimetype.
        # Set cookie value to 'syntax-error' so the next fetch will work in 'syntax-error' mode.
        content_type = 'text/html'
        response.set_cookie('mode', 'syntax-error');
    elif mode == 'syntax-error':
        # Set cookie value to 'throw-install' so the next fetch will work in 'throw-install' mode.
        content_type = 'application/javascript'
        response.set_cookie('mode', 'throw-install');
        extra_body = 'badsyntax(isbad;'
    elif mode == 'throw-install':
        # Unset and delete cookie to clean up the test setting.
        content_type = 'application/javascript'
        response.delete_cookie('mode')
        extra_body = "addEventListener('install', function(e) { throw new Error('boom'); });"

    headers.append(('Content-Type', content_type))
    # Return a different script for each access.  Use .time() and .clock() for
    # best time resolution across different platforms.
    return headers, '/* %s %s */ %s' % (time.time(), time.clock(), extra_body)

