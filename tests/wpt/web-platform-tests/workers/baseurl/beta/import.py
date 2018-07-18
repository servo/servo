def main(request, response):
    return (302, "Moved"), [("Location", "../gamma/import.js")], "postMessage('executed redirecting script');"

