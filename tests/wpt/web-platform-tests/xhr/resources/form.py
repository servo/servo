def main(request, response):
    return "id:%s;value:%s;" % (request.POST.first("id"), request.POST.first("value"))
