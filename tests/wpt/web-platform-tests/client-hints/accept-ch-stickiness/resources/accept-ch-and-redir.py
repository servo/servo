def main(request, response):
    url = ''
    if 'url' in request.GET:
        url = request.GET['url']
    return 301, [('Location', url),('Accept-CH', 'device-memory, DPR')], ''
