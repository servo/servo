def main(request, response):
  response.add_required_headers = False
  if b"content_type" in request.GET:
    response.writer.write_header(b"Content-Type", request.GET.first(b"content_type"))
  if b"nosniff" in request.GET:
  	response.writer.write_header(b"x-content-type-options", b"nosniff")
  response.writer.write_content(u"body { background:red }")
