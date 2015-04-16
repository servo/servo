def main(request, response):
    return int(request.GET["status"]), [], ""
