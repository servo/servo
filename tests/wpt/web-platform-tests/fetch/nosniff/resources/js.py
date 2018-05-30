def main(request, response):
    outcome = request.GET.first("outcome", "f")
    type = request.GET.first("type", "Content-Type missing")

    content = "// nothing to see here"
    content += "\n"
    content += "log('FAIL: " + type + "')" if (outcome == "f") else "p()"

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("x-content-type-options", "nosniff")
    response.writer.write_header("content-length", len(content))
    if(type != "Content-Type missing"):
        response.writer.write_header("content-type", type)
    response.writer.end_headers()

    response.writer.write(content)
