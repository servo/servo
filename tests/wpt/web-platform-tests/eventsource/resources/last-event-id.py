from six import ensure_str

def main(request, response):
  response.headers.set("Content-Type", "text/event-stream")

  last_event_id = ensure_str(request.headers.get("Last-Event-ID", ""))
  if last_event_id:
    return "data: " + last_event_id + "\n\n"
  else:
    idvalue = request.GET.first("idvalue", u"\u2026")
    return "id: " + idvalue + "\nretry: 200\ndata: hello\n\n"
