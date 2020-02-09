def main(request, response):
    return (
        (('Content-Type', 'text/javascript'),),
        'import "{}";\n'.format(request.GET.first('url'))
    )
