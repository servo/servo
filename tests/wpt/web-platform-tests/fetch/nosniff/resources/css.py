def main(request, response):
    type = request.GET.first("type", None)
    is_revalidation = request.headers.get("If-Modified-Since", None)

    content = "/* nothing to see here */"

    response.add_required_headers = False
    if is_revalidation is not None:
        response.writer.write_status(304)
        response.writer.write_header("x-content-type-options", "nosniff")
        response.writer.write_header("content-length", 0)
        if(type != None):
            response.writer.write_header("content-type", type)
        response.writer.end_headers()
        response.writer.write("")
    else:
        response.writer.write_status(200)
        response.writer.write_header("x-content-type-options", "nosniff")
        response.writer.write_header("content-length", len(content))
        if(type != None):
            response.writer.write_header("content-type", type)
        response.writer.end_headers()
        response.writer.write(content)
