import time

def main(request, response):
    time.sleep(float(request.GET.first(b"delay", 1000)) / 1000)
    response.headers.set('Content-Type', 'text/css')
    return "/* */"
