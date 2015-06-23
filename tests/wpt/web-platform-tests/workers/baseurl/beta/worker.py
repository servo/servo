def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/worker.js")], "postMessage('executed redirecting script');"

