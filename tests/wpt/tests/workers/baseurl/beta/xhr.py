def main(request, response):
    return (302, b"Moved"), [(b"Location", b"../gamma/xhr.js")], u"postMessage('executed redirecting script');"

