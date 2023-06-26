from wptserve.utils import isomorphic_encode

def main(request, response):
    content = []

    for key, values in sorted(item for item in request.POST.items() if not hasattr(item[1][0], u"filename")):
        content.append(b"%s=%s," % (key, values[0]))
    content.append(b"\n")

    for key, values in sorted(item for item in request.POST.items() if hasattr(item[1][0], u"filename")):
        value = values[0]
        content.append(b"%s=%s:%s:%d," % (key,
                                          isomorphic_encode(value.filename),
                                          isomorphic_encode(value.headers[u"Content-Type"]) if value.headers[u"Content-Type"] is not None else b"None",
                                          len(value.file.read())))

    return b"".join(content)
