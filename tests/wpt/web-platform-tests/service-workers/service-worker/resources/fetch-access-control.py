import base64
import json

def main(request, response):
    headers = []
    headers.append(('X-ServiceWorker-ServerHeader', 'SetInTheServer'))

    if "ACAOrigin" in request.GET:
        for item in request.GET["ACAOrigin"].split(","):
            headers.append(("Access-Control-Allow-Origin", item))

    for suffix in ["Headers", "Methods", "Credentials"]:
        query = "ACA%s" % suffix
        header = "Access-Control-Allow-%s" % suffix
        if query in request.GET:
            headers.append((header, request.GET[query]))

    if "ACEHeaders" in request.GET:
        headers.append(("Access-Control-Expose-Headers", request.GET["ACEHeaders"]))

    if ("Auth" in request.GET and not request.auth.username) or "AuthFail" in request.GET:
        status = 401
        headers.append(('WWW-Authenticate', 'Basic realm="Restricted"'))
        body = 'Authentication canceled'
        return status, headers, body

    if "PNGIMAGE" in request.GET:
        headers.append(("Content-Type", "image/png"))
        body = base64.decodestring("iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1B"
                                   "AACxjwv8YQUAAAAJcEhZcwAADsQAAA7EAZUrDhsAAAAhSURBVDhPY3wro/KfgQLABKXJBqMG"
                                   "jBoAAqMGDLwBDAwAEsoCTFWunmQAAAAASUVORK5CYII=")
        return headers, body


    username = request.auth.username if request.auth.username else "undefined"
    password = request.auth.password if request.auth.username else "undefined"
    cookie = request.cookies['cookie'].value if 'cookie' in request.cookies else "undefined"

    files = []
    for key, values in request.POST.iteritems():
        assert len(values) == 1
        value = values[0]
        if not hasattr(value, "file"):
            continue
        data = value.file.read()
        files.append({"key": key,
                      "name": value.file.name,
                      "type": value.type,
                      "error": 0, #TODO,
                      "size": len(data),
                      "content": data})

    get_data = {key:request.GET[key] for key,value in request.GET.iteritems()}
    post_data = {key:request.POST[key] for key,value in request.POST.iteritems()
                 if not hasattr(request.POST[key], "file")}
    headers_data = {key:request.headers[key] for key,value in request.headers.iteritems()}

    data = {"jsonpResult": "success",
            "method": request.method,
            "headers": headers_data,
            "body": request.body,
            "files": files,
            "GET": get_data,
            "POST": post_data,
            "username": username,
            "password": password,
            "cookie": cookie}

    return headers, "report( %s )" % json.dumps(data)
