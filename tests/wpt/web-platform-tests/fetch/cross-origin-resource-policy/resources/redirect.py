def main(request, response):
    headers = [("Location", request.GET['redirectTo'])]
    if 'corp' in request.GET:
        headers.append(('Cross-Origin-Resource-Policy', request.GET['corp']))

    return 302, headers, ""
