import base64
import json
import os
import sys

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

    if "VIDEO" in request.GET:
        headers.append(("Content-Type", "video/webm"))
        body = open(os.path.join(request.doc_root, "media", "movie_5.ogv"), "rb").read()
        length = len(body)
        # If "PartialContent" is specified, the requestor wants to test range
        # requests. For the initial request, respond with "206 Partial Content"
        # and don't send the entire content. Then expect subsequent requests to
        # have a "Range" header with a byte range. Respond with that range.
        if "PartialContent" in request.GET:
          if length < 1:
            return 500, headers, "file is too small for range requests"
          start = 0
          end = length - 1
          if "Range" in request.headers:
            range_header = request.headers["Range"]
            prefix = "bytes="
            split_header = range_header[len(prefix):].split("-")
            # The first request might be "bytes=0-". We want to force a range
            # request, so just return the first byte.
            if split_header[0] == "0" and split_header[1] == "":
              end = start
            # Otherwise, it is a range request. Respect the values sent.
            if split_header[0] != "":
              start = int(split_header[0])
            if split_header[1] != "":
              end = int(split_header[1])
          else:
            # The request doesn't have a range. Force a range request by
            # returning the first byte.
            end = start

          headers.append(("Accept-Ranges", "bytes"))
          headers.append(("Content-Length", str(end -start + 1)))
          headers.append(("Content-Range", "bytes %d-%d/%d" % (start, end, length)))
          chunk = body[start:(end + 1)]
          return 206, headers, chunk
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
