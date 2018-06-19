def main(request, response):
    headers = [("Cross-Origin-Resource-Policy", request.GET['corp'])]
    if 'origin' in request.headers:
        headers.append(('Access-Control-Allow-Origin', request.headers['origin']))

    return 200, headers, ""
