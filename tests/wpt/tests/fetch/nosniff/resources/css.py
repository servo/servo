def main(request, response):
    type = request.GET.first(b"type", None)
    is_revalidation = request.headers.get(b"If-Modified-Since", None)

    content = b"/* nothing to see here */"

    response.add_required_headers = False
    if is_revalidation is not None:
        response.writer.write_status(304)
        response.writer.write_header(b"x-content-type-options", b"nosniff")
        response.writer.write_header(b"content-length", 0)
        if(type != None):
            response.writer.write_header(b"content-type", type)
        response.writer.end_headers()
        response.writer.write(b"")
    else:
        response.writer.write_status(200)
        response.writer.write_header(b"x-content-type-options", b"nosniff")
        response.writer.write_header(b"content-length", len(content))
        if(type != None):
            response.writer.write_header(b"content-type", type)
        response.writer.end_headers()
        response.writer.write(content)
