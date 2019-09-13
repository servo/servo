def main(request, response):
    status = request.GET.first('status')
    response.status = (status, "");

