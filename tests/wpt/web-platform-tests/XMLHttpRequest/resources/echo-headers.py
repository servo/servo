def main(request, response):
    response.writer.write_status(200)
    response.writer.write_header("Content-Type", "text/plain")
    response.writer.end_headers()
    response.writer.write(str(request.raw_headers))
    response.close_connection = True
