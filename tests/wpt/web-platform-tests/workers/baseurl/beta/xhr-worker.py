def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/xhr-worker.js")], "postMessage('executed redirecting script');"
