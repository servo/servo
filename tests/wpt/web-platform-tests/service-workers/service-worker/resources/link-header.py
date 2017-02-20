def main(request, response):
    if 'Link' in request.GET:
        return [('Link', request.GET['Link'])], ""
    return [], ""
