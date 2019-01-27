# /xhr/resources/conditional.py -- to fake a 304 response

def main(request, response):
    tag = request.GET.first("tag", None)
    redirect = request.GET.first("redirect", None)
    match = request.headers.get("If-None-Match", None)
    date = request.GET.first("date", "")
    modified = request.headers.get("If-Modified-Since", None)
    response.headers.set("Access-Control-Allow-Origin", "*");
    response.headers.set("Timing-Allow-Origin", "*");
    if tag:
        response.headers.set("ETag", '"%s"' % tag)
    elif date:
        response.headers.set("Last-Modified", date)
    if redirect:
        response.headers.set("Location", redirect)
        response.status = (302, "Moved")
        return ""

    if ((match is not None and match == tag) or
        (modified is not None and modified == date)):
        response.status = (304, "SUPERCOOL")
        return ""
    else:
        response.headers.set("Content-Type", "text/plain")
        return "MAYBE NOT"
