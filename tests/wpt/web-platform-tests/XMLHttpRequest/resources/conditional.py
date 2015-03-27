def main(request, response):
    tag = request.GET.first("tag", None)
    match = request.headers.get("If-None-Match", None)
    date = request.GET.first("date", "")
    modified = request.headers.get("If-Modified-Since", None)
    if tag:
        response.headers.set("ETag", '"%s"' % tag)
    elif date:
        response.headers.set("Last-Modified", date)

    if ((match is not None and match == tag) or
        (modified is not None and modified == date)):
        response.status = (304, "SUPERCOOL")
        return ""
    else:
        response.headers.set("Content-Type", "text/plain")
        return "MAYBE NOT"
