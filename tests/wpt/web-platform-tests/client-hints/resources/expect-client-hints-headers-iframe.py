def main(request, response):
    """
    Simple handler that returns an HTML response that passes when the required
    Client Hints are received as request headers.
    """
    values = [ "Device-Memory", "DPR", "Viewport-Width" ]

    result = "PASS"
    log = ""
    for value in values:
        should = (request.GET[value.lower()] == "true")
        present = request.headers.get(value.lower()) or request.headers.get(value)
        log += value + " " + str(should) + " " + str(present) +", "
        if (should and not present) or (not should and present):
            result = "FAIL " + value + " " + str(should) + " " + str(present)
            break

    response.headers.append("Access-Control-Allow-Origin", "*")
    body = "<script>console.log('" + log +"'); window.parent.postMessage('" + result + "', '*');</script>"

    response.content = body
