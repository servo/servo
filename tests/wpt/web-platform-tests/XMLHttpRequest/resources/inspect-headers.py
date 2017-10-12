def get_response(raw_headers, filter_value, filter_name):
    result = ""
    for line in raw_headers.headers:
        if line[-2:] != '\r\n':
            return "Syntax error: missing CRLF: " + line
        line = line[:-2]

        if ':' not in line:
            return "Syntax error: no colon found: " + line
        name, value = line.split(':', 1)
        if len(value) > 1 and value[0] == ' ':
            value = value[1:]

        if filter_value:
            if value == filter_value:
                result += name + ","
        elif name.lower() == filter_name:
            result += name + ": " + value + "\n";
    return result

def main(request, response):
    headers = []
    if "cors" in request.GET:
        headers.append(("Access-Control-Allow-Origin", "*"))
        headers.append(("Access-Control-Allow-Credentials", "true"))
        headers.append(("Access-Control-Allow-Methods", "GET, POST, PUT, FOO"))
        headers.append(("Access-Control-Allow-Headers", "x-test, x-foo"))
        headers.append(("Access-Control-Expose-Headers", "x-request-method, x-request-content-type, x-request-query, x-request-content-length"))
    headers.append(("content-type", "text/plain"))

    filter_value = request.GET.first("filter_value", "")
    filter_name = request.GET.first("filter_name", "").lower()
    result = get_response(request.raw_headers, filter_value, filter_name)

    return headers, result
