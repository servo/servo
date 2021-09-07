import json
import os
from base64 import decodebytes

from wptserve.utils import isomorphic_decode, isomorphic_encode

def main(request, response):
    headers = []
    headers.append((b'X-ServiceWorker-ServerHeader', b'SetInTheServer'))

    if b"ACAOrigin" in request.GET:
        for item in request.GET[b"ACAOrigin"].split(b","):
            headers.append((b"Access-Control-Allow-Origin", item))

    for suffix in [b"Headers", b"Methods", b"Credentials"]:
        query = b"ACA%s" % suffix
        header = b"Access-Control-Allow-%s" % suffix
        if query in request.GET:
            headers.append((header, request.GET[query]))

    if b"ACEHeaders" in request.GET:
        headers.append((b"Access-Control-Expose-Headers", request.GET[b"ACEHeaders"]))

    if (b"Auth" in request.GET and not request.auth.username) or b"AuthFail" in request.GET:
        status = 401
        headers.append((b'WWW-Authenticate', b'Basic realm="Restricted"'))
        body = b'Authentication canceled'
        return status, headers, body

    if b"PNGIMAGE" in request.GET:
        headers.append((b"Content-Type", b"image/png"))
        body = decodebytes(b"iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1B"
                           b"AACxjwv8YQUAAAAJcEhZcwAADsQAAA7EAZUrDhsAAAAhSURBVDhPY3wro/KfgQLABKXJBqMG"
                           b"jBoAAqMGDLwBDAwAEsoCTFWunmQAAAAASUVORK5CYII=")
        return headers, body

    if b"VIDEO" in request.GET:
        headers.append((b"Content-Type", b"video/webm"))
        body = open(os.path.join(request.doc_root, u"media", u"movie_5.ogv"), "rb").read()
        length = len(body)
        # If "PartialContent" is specified, the requestor wants to test range
        # requests. For the initial request, respond with "206 Partial Content"
        # and don't send the entire content. Then expect subsequent requests to
        # have a "Range" header with a byte range. Respond with that range.
        if b"PartialContent" in request.GET:
          if length < 1:
            return 500, headers, b"file is too small for range requests"
          start = 0
          end = length - 1
          if b"Range" in request.headers:
            range_header = request.headers[b"Range"]
            prefix = b"bytes="
            split_header = range_header[len(prefix):].split(b"-")
            # The first request might be "bytes=0-". We want to force a range
            # request, so just return the first byte.
            if split_header[0] == b"0" and split_header[1] == b"":
              end = start
            # Otherwise, it is a range request. Respect the values sent.
            if split_header[0] != b"":
              start = int(split_header[0])
            if split_header[1] != b"":
              end = int(split_header[1])
          else:
            # The request doesn't have a range. Force a range request by
            # returning the first byte.
            end = start

          headers.append((b"Accept-Ranges", b"bytes"))
          headers.append((b"Content-Length", isomorphic_encode(str(end -start + 1))))
          headers.append((b"Content-Range", b"bytes %d-%d/%d" % (start, end, length)))
          chunk = body[start:(end + 1)]
          return 206, headers, chunk
        return headers, body

    username = request.auth.username if request.auth.username else b"undefined"
    password = request.auth.password if request.auth.username else b"undefined"
    cookie = request.cookies[b'cookie'].value if b'cookie' in request.cookies else b"undefined"

    files = []
    for key, values in request.POST.items():
        assert len(values) == 1
        value = values[0]
        if not hasattr(value, u"file"):
            continue
        data = value.file.read()
        files.append({u"key": isomorphic_decode(key),
                      u"name": value.file.name,
                      u"type": value.type,
                      u"error": 0, #TODO,
                      u"size": len(data),
                      u"content": data})

    get_data = {isomorphic_decode(key):isomorphic_decode(request.GET[key]) for key, value in request.GET.items()}
    post_data = {isomorphic_decode(key):isomorphic_decode(request.POST[key]) for key, value in request.POST.items()
                 if not hasattr(request.POST[key], u"file")}
    headers_data = {isomorphic_decode(key):isomorphic_decode(request.headers[key]) for key, value in request.headers.items()}

    data = {u"jsonpResult": u"success",
            u"method": request.method,
            u"headers": headers_data,
            u"body": isomorphic_decode(request.body),
            u"files": files,
            u"GET": get_data,
            u"POST": post_data,
            u"username": isomorphic_decode(username),
            u"password": isomorphic_decode(password),
            u"cookie": isomorphic_decode(cookie)}

    return headers, u"report( %s )" % json.dumps(data)
