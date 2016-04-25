def main(request, response):
    if request.POST.first('foo') == 'bar':
        return 'OK'
    else:
        return 'FAIL'
