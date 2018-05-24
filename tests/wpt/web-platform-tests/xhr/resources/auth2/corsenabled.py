import imp
import os

here = os.path.dirname(__file__)


def main(request, response):
    response.headers.set('Access-Control-Allow-Origin', request.headers.get("origin"))
    response.headers.set('Access-Control-Allow-Credentials', 'true')
    response.headers.set('Access-Control-Allow-Methods', 'GET')
    response.headers.set('Access-Control-Allow-Headers', 'authorization, x-user, x-pass')
    response.headers.set('Access-Control-Expose-Headers', 'x-challenge, xhr-user, ses-user')
    auth = imp.load_source("", os.path.abspath(os.path.join(here, os.pardir, "authentication.py")))
    if request.method == "OPTIONS":
        return ""
    else:
        return auth.main(request, response)
