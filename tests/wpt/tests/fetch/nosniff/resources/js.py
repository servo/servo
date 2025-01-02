def main(request, response):
    outcome = request.GET.first(b"outcome", b"f")
    type = request.GET.first(b"type", b"Content-Type missing")

    content = b"// nothing to see here"
    content += b"\n"
    content += b"log('FAIL: " + type + b"')" if (outcome == b"f") else b"p()"

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"x-content-type-options", b"nosniff")
    response.writer.write_header(b"content-length", len(content))
    if(type != b"Content-Type missing"):
        response.writer.write_header(b"content-type", type)
    response.writer.end_headers()

    response.writer.write(content)
