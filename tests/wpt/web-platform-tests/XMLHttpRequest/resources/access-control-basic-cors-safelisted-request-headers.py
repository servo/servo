def main(request, response):
    response.headers.set("Cache-Control", "no-store")

    # This should be a simple request; deny preflight
    if request.method != "POST":
        response.status = 400
        return

    response.headers.set("Access-Control-Allow-Credentials", "true")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))

    for header in ["Accept", "Accept-Language", "Content-Language", "Content-Type"]:
        value = request.headers.get(header)
        response.content += header + ": " + (value if value else "<None>") + '\n'
