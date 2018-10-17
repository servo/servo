def main(request, response):
    """
    Simple handler that sets a response header based on which client hint
    request headers were received.
    """

    response.headers.append("Access-Control-Allow-Origin", "*")
    values = request.GET
    name = values.first('name')
    type = values.first('mimeType')
    dpr = values.first('dpr')
    double = None
    if 'double' in values:
        double = values.first('double')
    image_path = request.doc_root + "/".join(request.url_parts[2].split("/")[:-1]) + "/" + name
    f = open(image_path, "rb")
    buff = f.read()
    f.close()
    response.headers.set("Content-Type", type)
    response.headers.set("Content-DPR", dpr)
    if double:
        response.headers.append("Content-DPR", double)
    response.headers.set("Content-Length", len(buff))
    response.content = buff
