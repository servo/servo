import os.path
import time

def main(request, response):
    name = request.GET.first("name")
    sleepTime = float(request.GET.first("sleep")) / 1E3

    time.sleep(sleepTime)

    path = os.path.join(os.path.dirname(__file__), name)
    body = open(path, "rb").read()

    response.headers.set("Content-Type", "image")
    response.headers.set("Content-Length", len(body))
    response.headers.set("Cache-control", "no-cache, must-revalidate")

    response.content = body;
