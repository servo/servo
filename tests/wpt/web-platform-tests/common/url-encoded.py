def main(request, response):
    if request.body == "foo=bara":
        return "OK"
    else:
        return "FAIL"
