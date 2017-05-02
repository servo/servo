def main(request, response):
    tag = request.GET.first("tag", None)
    match = request.headers.get("If-None-Match", None)
    date = request.GET.first("date", "")
    modified = request.headers.get("If-Modified-Since", None)
    cors = request.GET.first("cors", None)

    if request.method == "OPTIONS":
        response.headers.set("Access-Control-Allow-Origin", "*")
        response.headers.set("Access-Control-Allow-Headers", "IF-NONE-MATCH")
        return ""

    if tag:
        response.headers.set("ETag", '"%s"' % tag)
    elif date:
        response.headers.set("Last-Modified", date)

    if cors:
        response.headers.set("Access-Control-Allow-Origin", "*")

    if ((match is not None and match == tag) or
        (modified is not None and modified == date)):
        response.status = (304, "SUPERCOOL")
        return ""
    else:
        if not cors:
            response.headers.set("Access-Control-Allow-Origin", "*")
        response.headers.set("Content-Type", "text/plain")
        return "MAYBE NOT"
