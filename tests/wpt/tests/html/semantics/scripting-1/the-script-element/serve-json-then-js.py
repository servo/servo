# Respond with valid JSON to the first request with the given key,
# and with valid JavaScript to the second. Used for testing scenarios where
# the same request URL results in different responses on subsequent requests.
def main(request, response):
    try:
        stash_key = request.GET.first(b"key")

        run_count = request.server.stash.take(stash_key)
        if not run_count:
            run_count = 0

        if run_count == 0:
            response.headers.set(b"Content-Type", b"text/json")
            response.content = '{"hello": "world"}'
        else:
            response.headers.set(b"Content-Type", b"application/javascript")
            response.content = "export default 'hello';"

        request.server.stash.put(stash_key, run_count + 1)
    except:
        response.set_error(400, u"Not enough parameters")
