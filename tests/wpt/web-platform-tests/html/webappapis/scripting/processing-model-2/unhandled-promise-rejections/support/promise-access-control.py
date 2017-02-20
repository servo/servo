def main(request, response):
    allow = request.GET.first("allow", "false")

    headers = [("Content-Type", "application/javascript")]
    if allow != "false":
        headers.append(("Access-Control-Allow-Origin", "*"))

    body = "new Promise(function(resolve, reject) { reject(42); })"

    return headers, body
