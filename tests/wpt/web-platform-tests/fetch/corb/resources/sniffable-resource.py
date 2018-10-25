def main(request, response):
    body = request.GET.first("body", None)
    type = request.GET.first("type", None)

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("content-length", len(body))
    response.writer.write_header("content-type", type)
    response.writer.end_headers()

    response.writer.write(body)
