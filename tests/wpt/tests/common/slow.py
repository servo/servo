import time

def main(request, response):
    delay = float(request.GET.first(b"delay", 2000)) / 1000
    time.sleep(delay)
    return 200, [], b''
