import time

def main(request, response):
    delay = float(request.GET.first(b"ms", 500))
    time.sleep(delay / 1E3)

    return [(b"Content-type", b"text/javascript")], u"export let delayedLoaded = true;"
