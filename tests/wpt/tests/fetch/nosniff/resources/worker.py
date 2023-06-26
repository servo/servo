def main(request, response):
    type = request.GET.first(b"type", None)

    content = b"// nothing to see here"
    content += b"\n"
    content += b"this.postMessage('hi')"

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"x-content-type-options", b"nosniff")
    response.writer.write_header(b"content-length", len(content))
    if(type != None):
        response.writer.write_header(b"content-type", type)
    response.writer.end_headers()

    response.writer.write(content)
