def main(request, response):
    type = request.GET.first(b"type", None)
    subtype = request.GET.first(b"subtype", None)
    if type and subtype:
        response.headers[b"Content-Type"] = type + b"/" + subtype

    removeContentType = request.GET.first(b"removeContentType", None)
    if removeContentType:
        try:
            del response.headers[b"Content-Type"]
        except KeyError:
            pass

    content = b'<head>'
    mimeHead = request.GET.first(b"mime", None);
    if mimeHead:
        content += b'<meta http-equiv="Content-Type" content="%s; charset=utf-8"/>' % mimeHead
    content += b"</head>"

    return content
