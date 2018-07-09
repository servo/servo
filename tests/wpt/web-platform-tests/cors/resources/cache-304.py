def main(request, response):
    match = request.headers.get("If-None-Match", None)
    if match is not None and match == "mybestscript-v1":
        response.status = (304, "YEP")
        return ""
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Cache-Control", "must-revalidate")
    response.headers.set("ETag", "mybestscript-v1")
    response.headers.set("Content-Type", "text/javascript")
    return "function hep() { }"
