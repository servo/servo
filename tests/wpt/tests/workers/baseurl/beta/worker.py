def main(request, response):
    return (302, b"Moved"), [(b"Location", b"../gamma/worker.js")], u"postMessage('executed redirecting script');"

