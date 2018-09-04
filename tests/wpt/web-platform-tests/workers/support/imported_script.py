def main(request, response):
    return [('Content-Type', request.GET['mime'])], ""
