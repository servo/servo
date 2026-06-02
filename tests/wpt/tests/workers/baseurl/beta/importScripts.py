def main(request, response):
    return (302, b"Moved"), [(b"Location", b"../gamma/importScripts.js")], u"postMessage('executed redirecting script');"

