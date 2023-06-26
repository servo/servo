def main(request, response):
    token = request.GET.first(b"token")
    if request.server.stash.take(token) is not None:
        return b"1"
    else:
        return b"0"
