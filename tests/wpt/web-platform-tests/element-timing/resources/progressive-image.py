import os.path
import time

def main(request, response):
    name = request.GET.first("name")
    sleepTime = float(request.GET.first("sleep")) / 1E3
    numInitial = int(request.GET.first("numInitial"))

    path = os.path.join(os.path.dirname(__file__), name)
    body = open(path, "rb").read()

    response.headers.set("Content-Type", "image")
    response.headers.set("Content-Length", len(body))
    response.headers.set("Cache-control", "no-cache, must-revalidate")
    response.write_status_headers()

    # Read from the beginning, |numInitial| bytes.
    first = body[:numInitial]
    response.writer.write_content(first)
    response.writer.flush()

    time.sleep(sleepTime)

    # Read the remainder after having slept.
    second = body[numInitial:]
    response.writer.write_content(second)
