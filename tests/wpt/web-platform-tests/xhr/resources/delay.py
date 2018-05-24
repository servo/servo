import time

def main(request, response):
    delay = float(request.GET.first("ms", 500))
    time.sleep(delay / 1E3)

    return [("Access-Control-Allow-Origin", "*"), ("Access-Control-Allow-Methods", "YO"), ("Content-type", "text/plain")], "TEST_DELAY"
