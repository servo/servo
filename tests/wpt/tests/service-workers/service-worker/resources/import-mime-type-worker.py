def main(request, response):
    if b'mime' in request.GET:
        return (
            [(b'Content-Type', b'application/javascript')],
            b"importScripts('./mime-type-worker.py?mime=%s');" % request.GET[b'mime']
        )
    return (
        [(b'Content-Type', b'application/javascript')],
        b"importScripts('./mime-type-worker.py');"
    )
