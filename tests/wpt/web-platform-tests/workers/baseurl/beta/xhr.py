def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/xhr.js")], "postMessage('executed redirecting script');"

