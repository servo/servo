import time

def main(request, response):
    mime = request.GET.first("mime", "text/event-stream")
    message = request.GET.first("message", "data: data");
    newline = "" if request.GET.first("newline", None) == "none" else "\n\n";
    sleep = int(request.GET.first("sleep", "0"))

    headers = [("Content-Type", mime)]
    body = message + newline + "\n"
    if sleep != 0:
        time.sleep(sleep/1000)

    return headers, body
