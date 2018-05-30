from json import JSONEncoder, JSONDecoder
from base64 import b64decode

def main(request, response):
    uuid = request.GET.first("token", None)
    if "querystate" in request.GET:
        response.headers.set("Content-Type", "text/plain")
        return JSONEncoder().encode(request.server.stash.take(uuid))

    server_state = request.server.stash.take(uuid)
    if not server_state:
        server_state = []

    requests = JSONDecoder().decode(b64decode(request.GET.first("info", "")))
    config = requests[len(server_state)]

    state = dict()
    state["request_method"] = request.method
    state["request_headers"] = dict([[h.lower(), request.headers[h]] for h in request.headers])
    server_state.append(state)
    request.server.stash.put(uuid, server_state)

    note_headers = ['content-type', 'access-control-allow-origin', 'last-modified', 'etag']
    noted_headers = {}
    for header in config.get('response_headers', []):
        if header[0].lower() in ["location", "content-location"]: # magic!
            header[1] = "%s&target=%s" % (request.url, header[1])
        response.headers.set(header[0], header[1])
        if header[0].lower() in note_headers:
            noted_headers[header[0].lower()] = header[1]

    if "access-control-allow-origin" not in noted_headers:
        response.headers.set("Access-Control-Allow-Origin", "*")
    if "content-type" not in noted_headers:
        response.headers.set("Content-Type", "text/plain")
    response.headers.set("Server-Request-Count", len(server_state))

    code, phrase = config.get("response_status", [200, "OK"])

    if request.headers.get("If-Modified-Since", False) == noted_headers.get('last-modified', True):
        code, phrase = [304, "Not Modified"]
    if request.headers.get("If-None-Match", False) == noted_headers.get('etag', True):
        code, phrase = [304, "Not Modified"]

    response.status = (code, phrase)

    content = config.get("response_body", uuid)
    if code in [204, 304]:
        return ""
    else:
        return content
