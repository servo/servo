import imp
import os

def main(request, response):
    response.headers.set('Access-Control-Allow-Origin', request.headers.get("origin"));
    response.headers.set('Access-Control-Allow-Credentials', 'true');
    response.headers.set('Access-Control-Allow-Methods', 'GET');
    response.headers.set('Access-Control-Allow-Headers', 'authorization, x-user, x-pass');
    response.headers.set('Access-Control-Expose-Headers', 'x-challenge, xhr-user, ses-user');
    auth = imp.load_source("", os.path.join(os.path.abspath(os.curdir),
                                            "XMLHttpRequest",
                                            "resources",
                                            "authentication.py"))
    if request.method == "OPTIONS":
        return ""
    else:
        return auth.main(request, response)



