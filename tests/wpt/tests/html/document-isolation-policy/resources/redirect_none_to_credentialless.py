DIP_HEADER = b"|header(document-isolation-policy, isolate-and-credentialless)"

def main(request, response):
    """
    Causes a redirection that the initial response doesn't have credentialless and the
    redirected response has credentialless.
    """
    response.status = 302

    location = \
        request.GET[b'redirectTo'] + b"&pipe=" + DIP_HEADER

    response.headers.set(b"Location", location)
