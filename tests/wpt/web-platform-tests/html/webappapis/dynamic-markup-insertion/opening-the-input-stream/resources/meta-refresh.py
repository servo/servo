import time

def main(request, response):
    time = request.url_parts.query if request.url_parts.query else '0'
    return 200, [['Content-Type', 'text/html']], '<meta http-equiv=refresh content=%s>' % time
