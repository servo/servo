def main(request, response):
    if request.method == "POST":
        response.add_required_headers = False
        response.writer.write_status(302)
        response.writer.write_header("Location", request.url)
        response.writer.end_headers()
        response.writer.write("")
    elif request.method == "GET":
        return ([("Content-Type", "text/plain")],
                "OK")
    else:
        return ([("Content-Type", "text/plain")],
                "FAIL")