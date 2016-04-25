def main(request, response):
    if request.headers.get('Content-Type') == 'application/x-www-form-urlencoded':
        if request.body == 'foo=bara':
            return 'OK'
        else:
            return 'FAIL'
    else:
        if request.POST.first('foo') == 'bar':
            return 'OK'
        else:
            return 'FAIL'

