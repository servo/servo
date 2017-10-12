def main(request, response):
    if "base" in request.GET:
        return [("Content-Type", "text/html")], "OK"
    type = request.GET.first("type")

    if type == "normal":
        response.status = 302
        response.headers.append("Location", "redirect-redirected.html")
        response.headers.append("Custom-Header", "hello")
        return ""

    if type == "no-location":
        response.status = 302
        response.headers.append("Custom-Header", "hello")
        return ""

    if type == "no-location-with-body":
        response.status = 302
        response.headers.append("Content-Type", "text/html")
        response.headers.append("Custom-Header", "hello")
        return "<body>BODY</body>"

    if type == "redirect-to-scope":
        response.status = 302
        response.headers.append("Location",
                                "redirect-scope.py?type=redirect-to-scope2")
        return ""
    if type == "redirect-to-scope2":
        response.status = 302
        response.headers.append("Location",
                                "redirect-scope.py?type=redirect-to-scope3")
        return ""
    if type == "redirect-to-scope3":
        response.status = 302
        response.headers.append("Location", "redirect-redirected.html")
        response.headers.append("Custom-Header", "hello")
        return ""
