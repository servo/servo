COEP_HEADER = b"|header(cross-origin-embedder-policy, credentialless)"
CORP_HEADER = b"|header(cross-origin-resource-policy, cross-origin)"

def main(request, response):
    """
    Causes a redirection that the initial response doesn't have credentialless and the
    redirected response has credentialless.
    """
    response.status = 302

    location = \
        request.GET[b'redirectTo'] + b"&pipe=" + COEP_HEADER + CORP_HEADER

    response.headers.set(b"Location", location)
