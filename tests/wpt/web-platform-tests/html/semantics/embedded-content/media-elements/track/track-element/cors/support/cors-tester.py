from wptserve.handlers import HTTPException
import urllib

def main(request, response):
    if request.method != "GET":
        raise HTTPException(400, message="Method was not GET")

    if not "id" in request.GET:
        raise HTTPException(400, message="No id")

    id = request.GET['id']

    if "read" in request.GET:
        data = request.server.stash.take(id)
        if data is None:
            response.set_error(404, "Tried to read data not yet set")
            return
        return [("Content-Type", "text/plain")], data

    elif "cleanup" in request.GET:
        request.server.stash.take(id)
        return "OK"

    elif "delete-cookie" in request.GET:
        response.delete_cookie(id)
        return [("Content-Type", "text/plain")], "OK"

    if "origin" in request.GET:
        response.headers.set('Access-Control-Allow-Origin', request.GET['origin'])
        response.headers.set('Access-Control-Allow-Credentials', 'true')

    cors = request.headers.get("origin", "no")

    cookie = request.cookies.first(id, "no")

    line = 'cors = ' + cors + ' | cookie = ' + cookie.value;

    data = request.server.stash.take(id)
    if data is not None:
        line = data + "\n" + line

    request.server.stash.put(id, line)

    if "redirect" in request.GET:
        response.status = 302
        response.headers.set('Location', request.GET['redirect'])
    else:
        return """WEBVTT

00:00:00.000 --> 00:00:10.000
Test"""
