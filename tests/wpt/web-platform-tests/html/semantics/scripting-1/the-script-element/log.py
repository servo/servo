import time

def main(request, response):
    response.headers.append("Content-Type", "text/javascript")
    try:
        script_id = int(request.GET.first("id"))
        delay = int(request.GET.first("sec"))
    except:
        response.set_error(400, "Invalid parameter")

    time.sleep(int(delay))

    return "log('%s')" % script_id
