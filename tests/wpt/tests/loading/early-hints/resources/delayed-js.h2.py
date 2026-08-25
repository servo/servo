import importlib

utils = importlib.import_module("loading.early-hints.resources.utils")


def main(request, response):
    id = request.GET.first(b"id")
    # Wait until the id is set via resume-delayed-js.h2.py.
    utils.wait_for_preload_to_finish(request, id)

    headers = [
        ("Content-Type", "text/javascript"),
        ("Cache-Control", "max-age=600"),
    ]
    body = "/*empty script*/"
    return (200, "OK"), headers, body
