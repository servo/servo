def main(request, response):
    type = request.GET['type']
    if request.GET['type'] == 'fallingback':
        return 404, [('Content-Type', 'text/plain')], "Page not found"
    return [('Content-Type', 'text/plain')], type
