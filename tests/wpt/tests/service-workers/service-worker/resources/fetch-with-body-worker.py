def main(request, response):
    if len(request.body):
        return 200, [], u"BODY"
    return 400, [], u"NO BODY"
