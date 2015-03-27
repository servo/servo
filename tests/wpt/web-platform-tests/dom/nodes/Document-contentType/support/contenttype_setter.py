def main(request, response):
    type = request.GET.first("type", None)
    subtype = request.GET.first("subtype", None)
    if type and subtype:
        response.headers["Content-Type"] = type + "/" + subtype

    removeContentType = request.GET.first("removeContentType", None)
    if removeContentType:
        try:
            del response.headers["Content-Type"]
        except KeyError:
            pass

    content = '<head>'
    mimeHead = request.GET.first("mime", None);
    if mimeHead:
        content += '<meta http-equiv="Content-Type" content="%s; charset=utf-8"/>' % mimeHead
    content += "</head>"

    return content
