def main(request, response):
    if 'mime' in request.GET:
        return (
            [('Content-Type', 'application/javascript')],
            "importScripts('./mime-type-worker.py?mime={0}');".format(request.GET['mime'])
        )
    return (
        [('Content-Type', 'application/javascript')],
        "importScripts('./mime-type-worker.py');"
    )
