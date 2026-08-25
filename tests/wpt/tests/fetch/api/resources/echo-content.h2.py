def handle_headers(frame, request, response):
    response.status = 200
    response.headers.update([('Content-Type', 'text/plain')])
    response.write_status_headers()

def handle_data(frame, request, response):
    response.writer.write_data(frame.data)
