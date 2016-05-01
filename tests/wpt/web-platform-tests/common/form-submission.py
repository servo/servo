def main(request, response):
    if request.headers.get('Content-Type') == 'application/x-www-form-urlencoded':
        if request.body == 'foo=bara':
            return 'OK'
        else:
            return 'FAIL'
    elif request.headers.get('Content-Type') == 'text/plain':
        if request.body == 'qux=baz\r\n':
            return 'OK'
        else:
            return 'FAIL'
    else:
        if request.POST.first('foo') == 'bar':
            return 'OK'
        else:
            return 'FAIL'

