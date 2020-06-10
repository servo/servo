def main(request, response):
    match = request.headers.get(b"If-None-Match", None)
    if match is not None and match == b"mybestscript-v1":
        response.status = (304, u"YEP")
        return u""
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Cache-Control", b"must-revalidate")
    response.headers.set(b"ETag", b"mybestscript-v1")
    response.headers.set(b"Content-Type", b"text/javascript")
    return u"function hep() { }"
