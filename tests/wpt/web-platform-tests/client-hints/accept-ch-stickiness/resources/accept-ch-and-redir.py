def main(request, response):
    url = b''
    if b'url' in request.GET:
        url = request.GET[b'url']
    return 301, [(b'Location', url),(b'Accept-CH', b'device-memory, DPR')], u''
