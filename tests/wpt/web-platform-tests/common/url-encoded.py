def main(request, response):
    if request.body == "foo=bara":
        return [("Content-Type", "text/plain")], "OK"
    else:
        return [("Content-Type", "text/plain")], "FAIL"
