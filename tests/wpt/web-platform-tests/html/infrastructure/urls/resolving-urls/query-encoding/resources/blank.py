def main(request, response):
    return [("Content-Type", "text/html; charset=%s" % (request.GET['encoding']))], ""
