def main(request, response):
    token = request.GET.first(b"token")
    if request.server.stash.remove(token) is not None:
        return u"1"
    else:
        return u"0"
