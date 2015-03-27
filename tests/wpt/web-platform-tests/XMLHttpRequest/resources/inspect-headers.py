def main(request, response):
    headers = []
    if "cors" in request.GET:
        headers.append(("Access-Control-Allow-Origin", "*"))
        headers.append(("Access-Control-Allow-Credentials", "true"))
        headers.append(("Access-Control-Allow-Methods", "GET, POST, PUT, FOO"))
        headers.append(("Access-Control-Allow-Headers", "x-test, x-foo"))
        headers.append(("Access-Control-Expose-Headers", "x-request-method, x-request-content-type, x-request-query, x-request-content-length"))

    filter_value = request.GET.first("filter_value", "")
    filter_name = request.GET.first("filter_name", "").lower()

    result = ""
    for name, value in request.headers.iteritems():
        if filter_value:
            if value == filter_value:
                result += name.lower() + ","
        elif name.lower() == filter_name:
            result += name.lower() + ": " + value + "\n";

    headers.append(("content-type", "text/plain"))
    return headers, result
