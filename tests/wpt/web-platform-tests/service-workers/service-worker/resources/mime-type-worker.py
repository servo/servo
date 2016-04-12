def main(request, response):
    if 'mime' in request.GET:
        return [('Content-Type', request.GET['mime'])], ""
    return [], ""
