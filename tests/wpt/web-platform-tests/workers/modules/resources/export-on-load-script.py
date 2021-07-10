import datetime

def main(request, response):
    # This script serves both preflight and main GET request for cross-origin
    # static imports from module service workers.
    # According to https://w3c.github.io/ServiceWorker/#update-algorithm,
    # `Service-Worker: script` request header is added, which triggers CORS
    # preflight.
    response_headers = [(b"Content-Type", b"text/javascript"),
                        (b"Access-Control-Allow-Origin", b"*"),
                        (b"Access-Control-Allow-Headers", b"Service-Worker")]

    body = b"export const importedModules = ['export-on-load-script.js'];"
    body += b"// %d" % datetime.datetime.now().timestamp()
    return (200, response_headers, body)
