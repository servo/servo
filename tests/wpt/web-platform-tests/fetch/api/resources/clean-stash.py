def main(request, response):
    token = request.GET.first("token")
    if request.server.stash.take(token) is not None:
        return "1"
    else:
        return "0"
