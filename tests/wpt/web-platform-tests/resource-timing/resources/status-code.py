def main(request, response):
    status = request.GET.first(b'status')
    response.status = (status, b"");

