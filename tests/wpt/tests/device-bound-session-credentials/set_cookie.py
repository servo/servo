import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

# The client will ask the server to set a cookie, because client code can't
# always do so itself (e.g. HttpOnly attribute).
def main(request, response):
    request_body = request.body.decode('utf-8')
    return (200, [("Set-Cookie", request_body)], "")
