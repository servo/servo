def main(request, response):
  response.add_required_headers = False
  if "content_type" in request.GET:
    response.writer.write_header("Content-Type", request.GET.first("content_type"))
  if "nosniff" in request.GET:
  	response.writer.write_header("x-content-type-options", "nosniff")
  response.writer.write_content("body { background:red }")
