def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/importScripts.js")], "postMessage('executed redirecting script');"

