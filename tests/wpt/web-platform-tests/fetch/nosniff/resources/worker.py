def main(request, response):
    type = request.GET.first("type", None)

    content = "// nothing to see here"
    content += "\n"
    content += "this.postMessage('hi')"

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("x-content-type-options", "nosniff")
    response.writer.write_header("content-length", len(content))
    if(type != None):
        response.writer.write_header("content-type", type)
    response.writer.end_headers()

    response.writer.write(content)
