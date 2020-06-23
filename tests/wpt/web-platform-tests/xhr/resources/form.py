def main(request, response):
    return b"id:%s;value:%s;" % (request.POST.first(b"id"), request.POST.first(b"value"))
