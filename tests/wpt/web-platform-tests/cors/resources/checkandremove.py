def main(request, response):
    token = request.GET.first("token")
    if request.server.stash.remove(token) is not None:
        return "1"
    else:
        return "0"
