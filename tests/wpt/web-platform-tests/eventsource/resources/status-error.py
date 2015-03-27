def main(request, response):
  status = (request.GET.first("status", "404"), "HAHAHAHA")
  headers = [("Content-Type", "text/event-stream")]
  return status, headers, "data: data\n\n"
