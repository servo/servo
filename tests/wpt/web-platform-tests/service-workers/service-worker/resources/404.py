# iframe does not fire onload event if the response's content-type is not
# text/plain or text/html so this script exists if you want to test a 404 load
# in an iframe.
def main(req, res):
    return 404, [(b'Content-Type', b'text/plain')], b"Page not found"
