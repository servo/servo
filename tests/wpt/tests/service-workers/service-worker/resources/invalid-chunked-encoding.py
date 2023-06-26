def main(request, response):
    return [(b"Content-Type", b"application/javascript"), (b"Transfer-encoding", b"chunked")], b"XX\r\n\r\n"
