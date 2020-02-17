import os


def main(request, response):
    stash_id = request.GET.first("id")
    if request.server.stash.take(stash_id) is not None:
        response.status = (404, "Not Found")
        response.headers.set("Content-Type", "text/plain")
        return "not found"
    request.server.stash.put(stash_id, True)

    path = os.path.join(os.path.dirname(__file__), "sxg", "sxg-prefetch-test.sxg")
    body = open(path, "rb").read()

    response.headers.set("Content-Type", "application/signed-exchange;v=b3")
    response.headers.set("X-Content-Type-Options", "nosniff")
    response.headers.set("Cache-Control", "public, max-age=600")

    return body.replace('XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX', stash_id)
