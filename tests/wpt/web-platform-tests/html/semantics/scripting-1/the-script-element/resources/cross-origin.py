def main(request, response):
    headers = [("Content-Type", "text/javascript")]
    milk = request.cookies.first("milk", None)

    if milk is None:
        return headers, "var included = false;"
    elif milk.value == "yes":
        return headers, "var included = true;"

    return headers, "var included = false;"
