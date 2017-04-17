def main(request, response):
    headers = []
    headers.append(("Access-Control-Allow-Origin", "*"))

    return headers, "{ \"result\": \"success\" }"
