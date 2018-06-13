import time

def main(request, response):
    code = int(request.GET.first("code", 302))
    location = request.GET.first("location", request.url_parts.path + "?followed")

    if "delay" in request.GET:
        delay = float(request.GET.first("delay"))
        time.sleep(delay / 1E3)

    if "followed" in request.GET:
        return [("Content:Type", "text/plain")], "MAGIC HAPPENED"
    else:
        return (code, "WEBSRT MARKETING"), [("Location", location)], "TEST"
