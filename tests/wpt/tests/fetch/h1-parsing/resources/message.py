def main(request, response):
    response.writer.write(request.GET.first(b"message"))
    response.close_connection = True
