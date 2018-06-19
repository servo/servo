def main(request, response):
    headers = [("Content-Type", "text/html"),
               ("Cross-Origin-Resource-Policy", request.GET['corp'])]
    return 200, headers, "<body><h3>The iframe</h3><script>window.onmessage = () => { parent.postMessage('pong', '*'); }</script></body>"

