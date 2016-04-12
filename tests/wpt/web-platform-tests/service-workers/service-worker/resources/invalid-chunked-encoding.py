def main(request, response):
    return [("Content-Type", "application/javascript"), ("Transfer-encoding", "chunked")], "XX\r\n\r\n"
