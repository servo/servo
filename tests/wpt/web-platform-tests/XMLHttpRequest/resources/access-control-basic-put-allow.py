def main(request, response):
    if request.method == "OPTIONS":
        response.headers.set("Content-Type", "text/plain")
        response.headers.set("Access-Control-Allow-Credentials", "true")
        response.headers.set("Access-Control-Allow-Methods", "PUT")
        response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))

    elif request.method == "PUT":
        response.headers.set("Content-Type", "text/plain")
        response.headers.set("Access-Control-Allow-Credentials", "true")
        response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
        response.content = "PASS: Cross-domain access allowed."
        try:
            response.content += "\n" + request.body
        except:
            response.content += "Could not read in content."

    else:
        response.headers.set("Content-Type", "text/plain")
        response.content = "Wrong method: " + request.method
