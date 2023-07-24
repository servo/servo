import os


def main(request, response):
    response.headers.set(b"supports-loading-mode", b"fenced-frame")

    script = u"""
        <script src="utils.js"></script>
        <script>
            const [secfetch_key] = parseKeylist();
            writeValueToServer(secfetch_key, "%s")
        </script>
    """ % (request.headers.get(b"sec-fetch-dest", b"none"))
    return (200, [], script)
