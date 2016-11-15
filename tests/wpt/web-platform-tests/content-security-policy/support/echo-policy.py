def main(request, response):
    policy = request.GET.first("policy");
    return [("Content-Type", "text/html"), ("Content-Security-Policy", policy)], "<!DOCTYPE html><title>Echo.</title>"
