def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/sharedworker.js")], "postMessage('executed redirecting script');"

