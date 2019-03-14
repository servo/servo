import os.path

def main(request, response):
    etag = "123abc"
    if etag == request.headers.get("If-None-Match", None):
        response.headers.set("X-HTTP-STATUS", 304)
        response.status = (304, "Not Modified")
        return ""

    response.headers.set("Cache-Control", "public, max-age=86400")
    response.headers.set("Content-Type", "font/truetype")
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Timing-Allow-Origin", "*")
    response.headers.set("ETag", etag)
    font = "../../fonts/Ahem.ttf"
    path = os.path.join(os.path.dirname(__file__), font)
    response.content = open(path, "rb").read()
