import importlib
util = importlib.import_module('device-bound-session-credentials.verify_authenticated_util')

def main(request, response):
    return util.verify_authenticated(request, response)
