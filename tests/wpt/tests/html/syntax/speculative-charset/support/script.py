import time

def main(request, response):
    response.add_required_headers = False # Don't implicitly add HTTP headers
    response.writer.write_status(200)
    response.writer.write_header("Content-Type", "text/javascript")
    response.writer.end_headers()

    token = request.GET[b"uuid"]
    character = request.GET[b"character"]
    old_character = b"NOT LOADED PREVIOUSLY";
    from_stash = request.server.stash.take(token);
    if from_stash:
        old_character = from_stash
    else:
        request.server.stash.put(token, character)

    response.writer.write(b'parent.postMessage("token: %s, character: %s, previous character: %s, byte: \xE6", "*");' % (token, character, old_character));
